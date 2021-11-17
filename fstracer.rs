// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * fstracer - A filesystem-tracer
 * Copyright © 2021 rusty-snake
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::env::var_os;
use std::env::VarError;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Result as IoResult;
use std::io::Write;
use std::mem::transmute;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::FromRawFd;
use std::sync::Mutex;

use once_cell::sync::Lazy;

#[allow(unused_imports)]
#[rustfmt::skip]
use libc::{
    E2BIG, EACCES, EADDRINUSE, EADDRNOTAVAIL, EADV, EAFNOSUPPORT, EAGAIN, EALREADY, EBADE, EBADF,
    EBADFD, EBADMSG, EBADR, EBADRQC, EBADSLT, EBFONT, EBUSY, ECANCELED, ECHILD, ECHRNG, ECOMM,
    ECONNABORTED, ECONNREFUSED, ECONNRESET, EDEADLK, EDEADLOCK, EDESTADDRREQ, EDOM, EDOTDOT,
    EDQUOT, EEXIST, EFAULT, EFBIG, EHOSTDOWN, EHOSTUNREACH, EHWPOISON, EIDRM, EILSEQ, EINPROGRESS,
    EINTR, EINVAL, EIO, EISCONN, EISDIR, EISNAM, EKEYEXPIRED, EKEYREJECTED, EKEYREVOKED, EL2HLT,
    EL2NSYNC, EL3HLT, EL3RST, ELIBACC, ELIBBAD, ELIBEXEC, ELIBMAX, ELIBSCN, ELNRNG, ELOOP,
    EMEDIUMTYPE, EMFILE, EMLINK, EMSGSIZE, EMULTIHOP, ENAMETOOLONG, ENAVAIL, ENETDOWN, ENETRESET,
    ENETUNREACH, ENFILE, ENOANO, ENOBUFS, ENOCSI, ENODATA, ENODEV, ENOENT, ENOEXEC, ENOKEY, ENOLCK,
    ENOLINK, ENOMEDIUM, ENOMEM, ENOMSG, ENONET, ENOPKG, ENOPROTOOPT, ENOSPC, ENOSR, ENOSTR, ENOSYS,
    ENOTBLK, ENOTCONN, ENOTDIR, ENOTEMPTY, ENOTNAM, ENOTRECOVERABLE, ENOTSOCK, ENOTSUP, ENOTTY,
    ENOTUNIQ, ENXIO, EOPNOTSUPP, EOVERFLOW, EOWNERDEAD, EPERM, EPFNOSUPPORT, EPIPE, EPROTO,
    EPROTONOSUPPORT, EPROTOTYPE, ERANGE, EREMCHG, EREMOTE, EREMOTEIO, ERESTART, ERFKILL, EROFS,
    ESHUTDOWN, ESOCKTNOSUPPORT, ESPIPE, ESRCH, ESRMNT, ESTALE, ESTRPIPE, ETIME, ETIMEDOUT,
    ETOOMANYREFS, ETXTBSY, EUCLEAN, EUNATCH, EUSERS, EWOULDBLOCK, EXDEV, EXFULL,

    O_APPEND, O_ASYNC, O_CLOEXEC, O_CREAT, O_DIRECT, O_DIRECTORY, O_DSYNC, O_EXCL, O_LARGEFILE,
    O_NDELAY, O_NOATIME, O_NOCTTY, O_NOFOLLOW, O_NONBLOCK, O_PATH, O_RDONLY, O_RDWR, O_RSYNC,
    O_SYNC, O_TMPFILE, O_TRUNC, O_WRONLY,

    RTLD_NEXT,

    S_IRGRP, S_IROTH, S_IRUSR, S_IRWXG, S_IRWXO, S_IRWXU, S_ISGID, S_ISUID, S_ISVTX, S_IWGRP,
    S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR,

    __errno_location, dlsym,

    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort,

    mode_t,

    FILE,
};

macro_rules! cstr {
    ($bytes:literal) => {
        ::std::ffi::CStr::from_bytes_with_nul($bytes)
            .unwrap()
            .as_ptr()
    };
}

macro_rules! catch_unwind {
    { $( $tt:tt )* } => {
        ::std::panic::catch_unwind(|| { $( $tt )* }).unwrap_or_else(|_| unsafe {
            const ERROR_MESSAGE: &[u8] = b"fstracer: An error occurred, aborting ...\n";
            ::libc::write(
                ::libc::STDERR_FILENO,
                ERROR_MESSAGE as *const _ as *const ::libc::c_void,
                ERROR_MESSAGE.len(),
            );
            ::libc::abort();
        })
    };
}

static FSTRACER_OUTPUT: Lazy<Mutex<File>> = Lazy::new(|| {
    var_os("FSTRACER_OUTPUT")
        .ok_or_else(|| IoError::new(IoErrorKind::Other, VarError::NotPresent))
        .and_then(|fstracer_output| unsafe {
            let fstracer_output = CString::new(fstracer_output.into_vec())?;
            // Directly call open to avoid deadlocks we would have with functions like
            // File::create which call libc::open which is crate::open which would wait until we
            // are initialized.
            let fd = libc::syscall(
                libc::SYS_open,
                fstracer_output.as_ptr(),
                O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC,
                0o644,
            ) as c_int;
            if fd < 0 {
                Err(IoError::last_os_error())
            } else {
                Ok(File::from_raw_fd(fd))
            }
        })
        .map(Mutex::new)
        .expect("Failed to init FSTRACER_OUTPUT")
});

fn log(path: &[u8]) -> IoResult<()> {
    let mut fstracer_output = FSTRACER_OUTPUT.lock().unwrap();
    fstracer_output.write_all(path)?;
    fstracer_output.write_all(b"\n")?;
    Ok(())
}

/*
use std::sync::Once;
#[used]
#[link_section = ".init_array"]
static FSTRACER_INIT_FN: extern "C" fn() = init;
extern "C" fn init() {
    catch_unwind! {
        static FSTRACER_INIT: Once = Once::new();
        FSTRACER_INIT.call_once(|| {
        });
    }
}
#[used]
#[link_section = ".fini_array"]
static FSTRACER_FINI_FN: extern "C" fn() = fini;
extern "C" fn fini() {
    catch_unwind! {
        static FSTRACER_FINI: Once = Once::new();
        FSTRACER_FINI.call_once(|| {
        });
    }
}
*/

// Use C-compatible variadic functions (RFC 2137) instead of MaybeUninit when they are stable.
// RFC 2137: https://github.com/rust-lang/rfcs/blob/master/text/2137-variadic.md
// Tracking issue for RFC 2137: https://github.com/rust-lang/rust/issues/44930

#[no_mangle]
pub unsafe extern "C" fn open(
    path: *const c_char,
    oflag: c_int,
    mode: MaybeUninit<mode_t>,
) -> c_int {
    catch_unwind! {
        static REAL_OPEN: Lazy<extern "C" fn(*const c_char, c_int, ...) -> c_int> =
            Lazy::new(|| unsafe {
                let real_open = dlsym(RTLD_NEXT, cstr!(b"open\0"));
                assert!(!real_open.is_null());
                transmute(real_open)
            });

        log(CStr::from_ptr(path).to_bytes()).unwrap();

        let mode: mode_t = if oflag & (O_CREAT | O_TMPFILE) > 0 {
            mode.assume_init()
        } else {
            0
        };
        assert!(mode <= S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO);

        REAL_OPEN(path, oflag, mode)
    }
}

#[no_mangle]
pub unsafe extern "C" fn open64(
    path: *const c_char,
    oflag: c_int,
    mode: MaybeUninit<mode_t>,
) -> c_int {
    catch_unwind! {
        static REAL_OPEN64: Lazy<extern "C" fn(*const c_char, c_int, ...) -> c_int> =
            Lazy::new(|| unsafe {
                let real_open64 = dlsym(RTLD_NEXT, cstr!(b"open64\0"));
                assert!(!real_open64.is_null());
                transmute(real_open64)
            });

        log(CStr::from_ptr(path).to_bytes()).unwrap();

        let mode: mode_t = if oflag & (O_CREAT | O_TMPFILE) > 0 {
            mode.assume_init()
        } else {
            0
        };
        assert!(mode <= S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO);

        REAL_OPEN64(path, oflag, mode)
    }
}

#[no_mangle]
pub unsafe extern "C" fn openat(
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mode: MaybeUninit<mode_t>,
) -> c_int {
    catch_unwind! {
        static REAL_OPENAT: Lazy<extern "C" fn(c_int, *const c_char, c_int, ...) -> c_int> =
            Lazy::new(|| unsafe {
                let real_openat = dlsym(RTLD_NEXT, cstr!(b"openat\0"));
                assert!(!real_openat.is_null());
                transmute(real_openat)
            });

        log(CStr::from_ptr(pathname).to_bytes()).unwrap();

        let mode: mode_t = if flags & (O_CREAT | O_TMPFILE) > 0 {
            mode.assume_init()
        } else {
            0
        };
        assert!(mode <= S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO);

        REAL_OPENAT(dirfd, pathname, flags, mode)
    }
}

#[no_mangle]
pub unsafe extern "C" fn openat64(
    fd: c_int,
    path: *const c_char,
    oflag: c_int,
    mode: MaybeUninit<mode_t>,
) -> c_int {
    catch_unwind! {
        static REAL_OPENAT64: Lazy<extern "C" fn(c_int, *const c_char, c_int, ...) -> c_int> =
            Lazy::new(|| unsafe {
                let real_openat64 = dlsym(RTLD_NEXT, cstr!(b"openat64\0"));
                assert!(!real_openat64.is_null());
                transmute(real_openat64)
            });

        log(CStr::from_ptr(path).to_bytes()).unwrap();

        let mode: mode_t = if oflag & (O_CREAT | O_TMPFILE) > 0 {
            mode.assume_init()
        } else {
            0
        };
        assert!(mode <= S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO);

        REAL_OPENAT64(fd, path, oflag, mode)
    }
}

#[no_mangle]
pub unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    catch_unwind! {
        static REAL_FOPEN: Lazy<extern "C" fn(*const c_char, *const c_char) -> *mut FILE> =
            Lazy::new(|| unsafe {
                let real_fopen = dlsym(RTLD_NEXT, cstr!(b"fopen\0"));
                assert!(!real_fopen.is_null());
                transmute(real_fopen)
            });

        log(CStr::from_ptr(filename).to_bytes()).unwrap();

        REAL_FOPEN(filename, mode)
    }
}

#[no_mangle]
pub unsafe extern "C" fn fopen64(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    catch_unwind! {
        static REAL_FOPEN64: Lazy<extern "C" fn(*const c_char, *const c_char) -> *mut FILE> =
            Lazy::new(|| unsafe {
                let real_fopen64 = dlsym(RTLD_NEXT, cstr!(b"fopen64\0"));
                assert!(!real_fopen64.is_null());
                transmute(real_fopen64)
            });

        log(CStr::from_ptr(filename).to_bytes()).unwrap();

        REAL_FOPEN64(filename, mode)
    }
}
