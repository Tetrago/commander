use crate::monitor_button::MonitorButton;
use adw::prelude::PreferencesGroupExt;
use gtk::glib;
use hyprland::data::Monitors;
use hyprland::shared::HyprData;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "ui/display_group.blp")]
    pub struct DisplayGroup {}

    #[glib::object_subclass]
    impl ObjectSubclass for DisplayGroup {
        const NAME: &'static str = "DisplayGroup";
        type Type = super::DisplayGroup;
        type ParentType = adw::PreferencesGroup;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DisplayGroup {
        fn constructed(&self) {
            self.parent_constructed();

            for monitor in Monitors::get().expect("Failed to get Hyprland monitors") {
                self.obj().add(&MonitorButton::new(&monitor));
            }
        }
    }

    impl WidgetImpl for DisplayGroup {}
    impl PreferencesGroupImpl for DisplayGroup {}
}

glib::wrapper! {
    pub struct DisplayGroup(ObjectSubclass<imp::DisplayGroup>)
        @extends adw::PreferencesGroup, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
