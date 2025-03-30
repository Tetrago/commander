conf = configuration_data()
conf.set_quoted('APPLICATION_ID', application_id)
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf,
)

run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true,
)

sources = cargo_sources
sources += gresource

sources += files(
  'main.rs',
  'audio_devices_expander.rs',
  'window.rs',
)

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo, 'build', '--release',
    '--target-dir', meson.project_build_root() / 'target',
    '&&',
    'cp', meson.project_build_root() / 'target' / 'release' / meson.project_name(), '@OUTPUT@',
  ]
)
