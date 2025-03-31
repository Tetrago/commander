use adw::prelude::*;
use gtk::{gio, glib};

mod audio;
mod audio_device_row;
mod audio_devices_expander;
mod config;
mod window;

use config::{APPLICATION_ID, PKGDATADIR};

fn main() -> glib::ExitCode {
    gio::resources_register(
        &gio::Resource::load(PKGDATADIR.to_owned() + "/commander.gresource")
            .expect("Unable to find commander.gresource"),
    );

    let mut dev = audio::Audio::new();
    for device in dev.get_devices() {
        println!("{:?}", device);
    }
    drop(dev);

    audio_devices_expander::AudioDevicesExpander::static_type();

    let app = adw::Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = window::Window::new(app);
    window.present();
}
