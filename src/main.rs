use adw::prelude::*;
use gtk::glib;

mod audio;
mod audio_device_row;
mod audio_devices_expander;
mod display_group;
mod monitor_button;
mod window;

fn main() -> glib::ExitCode {
    audio_devices_expander::AudioDevicesExpander::static_type();
    display_group::DisplayGroup::static_type();

    let app = adw::Application::builder()
        .application_id("io.github.tetrago.commander")
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = window::Window::new(app);
    window.present();
}
