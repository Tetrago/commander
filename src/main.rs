use adw::prelude::*;
use gtk::glib;

mod audio;
mod monitors;
mod window;

fn main() -> glib::ExitCode {
    audio::AudioDevicesExpander::static_type();
    monitors::DisplaySidebarGroup::static_type();

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
