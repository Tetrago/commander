project('commander', 'rust', 'c',
  version: '0.1.0',
  meson_version: '>= 0.59.0'
)

dependency('gtk4', version: '>= 4.16.0')
dependency('libadwaita-1', version: '>= 1.5')

gnome = import('gnome')

blueprint_compiler = find_program('blueprint-compiler', required: true)
cargo = find_program('cargo', required: true)

application_id = 'io.github.tetrago.commander'
pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()

cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock',
)

subdir('data')
subdir('src')
