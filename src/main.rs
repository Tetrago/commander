use adw::prelude::*;
use gtk::glib;

mod audio;
mod audio_device_row;
mod audio_devices_expander;
mod window;

fn main() -> glib::ExitCode {
    let mut dev = audio::Audio::new();
    for device in dev.get_devices() {
        println!("{:?}", device);
    }
    drop(dev);

    audio_devices_expander::AudioDevicesExpander::static_type();

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
