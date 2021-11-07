bash = find_program('bash')

# Just to put a note somewhere how to get external crates
# - Add them as git submodules
# - Add them via cargo vendor
# - Add them with a custom script that parses cargo metadata
# - Add them with a custom script using https://crates.io/api/v1/crates/<name>

# -- libc --

rust_args_libc = [
  '--cap-lints', 'allow',

  '--cfg', 'feature="default"',
  # enabled by default
  '--cfg', 'feature="std"',
  # enabled by build.rs for non-FreeBSD systems
  '--cfg', 'feature="freebsd11"',
  # enabled by build.rs for rust >= 1.15
  '--cfg', 'feature="libc_priv_mod_use"',
  # enabled by build.rs for rust >= 1.19
  '--cfg', 'feature="libc_union"',
  # enabled by build.rs for rust >= 1.24
  '--cfg', 'feature="libc_const_size_of"',
  # enabled by build.rs for rust >= 1.25
  '--cfg', 'feature="libc_align"',
  # enabled by build.rs for rust >= 1.30
  '--cfg', 'feature="libc_core_cvoid"',
  # enabled by build.rs for rust >= 1.33
  '--cfg', 'feature="libc_packedN"',
  # enabled by build.rs for rust >= 1.33
  '--cfg', 'feature="libc_cfg_target_vendor"',
]

libc_sources = run_command(
  bash, '-O', 'globstar', '-c', 'echo libc/src/**/*.rs',
  check: true,
).stdout().split()

libc = static_library('libc', 'libc/src/lib.rs',
  rust_args: rust_args_libc,
  override_options: ['rust_std=2015'],
  sources: libc_sources,
)

# -- once_cell --

rust_args_once_cell = [
  '--cap-lints', 'allow',

  '--cfg', 'feature="default"',
  # enabled by default
  '--cfg', 'feature="std"',
  # enabled by std
  '--cfg', 'feature="alloc"',
  # enabled by alloc
  '--cfg', 'feature="race"',
]

once_cell_source = run_command(
  bash, '-O', 'globstar', '-c', 'echo once_cell/src/**/*.rs',
  check: true,
).stdout().split()

once_cell = static_library('once_cell', 'once_cell/src/lib.rs',
  rust_args: rust_args_once_cell,
  override_options: ['rust_std=2018'],
  sources: once_cell_source,
)
