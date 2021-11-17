# fstracer

A filesystem-tracer.

Actually this is just an experiment of mine to learn more about Rust's Foreign Function Interface (FFI), writing LD_PRELOAD libraries in Rust and meson's (experimental) Rust support.
You should not use it for production. Not all filesystem functions are implemented and the most code is marked with `unsafe`.

## Prerequisites

- [Rust] 1.56
- [meson] 0.60

[Rust]: https://www.rust-lang.org/
[meson]: https://mesonbuild.com/

## Compile

```bash
git clone "https://github.com/rusty-snake/fstracer.git"
RUSTC="$PWD/rustc" meson _builddir -Dbuildtype=release
meson compile -C _builddir
```

## Usage

```
$ FSTRACER_OUTPUT=/tmp/fstracer-celluloid.txt LD_PRELOAD=_builddir/libfstracer.so.0.1.0 celluloid
```

## License

GPL-3.0-or-later
