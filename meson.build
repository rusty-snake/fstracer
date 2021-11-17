# SPDX-License-Identifier: GPL-3.0-or-later

project('fstracer', 'rust', 'c',
  license: 'GPL-3.0-or-later',
  meson_version: '>=0.60.0',
  version: '0.1.0',
)

assert(meson.get_compiler('rust').version().version_compare('>=1.56'))

if get_option('buildtype') == 'release'
  add_project_arguments('-Ctarget-cpu=native', language: 'rust')
endif

# # # # # # # # # #

subdir('dependencies')

rust_args_fstracer = [
  #'-Clink-dead-code',
]

fstracer = shared_library('fstracer', 'fstracer.rs',
  rust_args: [rust_args_fstracer],
  link_with: [once_cell, libc],
  override_options: ['rust_std=2021'],
  rust_crate_type: 'cdylib',
  version: meson.project_version(),
)

subdir('tests')

# # # # # # # # # #

cargo = find_program('cargo', required: false)
if cargo.found()
  run_target('doc',
    command: [
      cargo, 'doc', '--lib', '--document-private-items',
      '--manifest-path=@CURRENT_SOURCE_DIR@/Cargo.toml',
    ],
  )
  run_target('update-dependencies',
    command: [
      cargo, 'vendor', '--no-delete', '@SOURCE_ROOT@/dependencies',
    ],
  )
endif

rustfmt = find_program('rustfmt', required: false)
if rustfmt.found()
  run_target('fmt',
    command: [
      rustfmt, '--edition=2021', files('fstracer.rs'),
    ],
  )
endif
