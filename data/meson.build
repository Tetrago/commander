blueprints = custom_target(
  'blueprints',
  input: files(
    'ui/audio_devices_expander.blp',
    'ui/audio_device_row.blp',
    'ui/window.blp',
  ),
  output: '.',
  command: [
    blueprint_compiler,
    'batch-compile',
    '@OUTDIR@',
    '@CURRENT_SOURCE_DIR@',
    '@INPUT@'
  ]
)

gresource = gnome.compile_resources(
  'commander',
  'commander.gresource.xml',
  dependencies: blueprints,
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir
)
