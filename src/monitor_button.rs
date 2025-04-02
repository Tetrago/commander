use gtk::glib;
use hyprland::data::Monitor;
use std::cell::RefCell;

mod imp {
    use super::*;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::MonitorButton)]
    #[template(file = "ui/monitor_button.blp")]
    pub struct MonitorButton {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[property(get, construct_only)]
        pub monitor_name: RefCell<String>,

        #[property(get, construct_only)]
        pub monitor_description: RefCell<String>,
    }

    #[gtk::template_callbacks]
    impl MonitorButton {
        #[template_callback]
        fn on_selected(&self) {}
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonitorButton {
        const NAME: &'static str = "MonitorButton";
        type Type = super::MonitorButton;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonitorButton {}

    impl WidgetImpl for MonitorButton {}
    impl ListBoxRowImpl for MonitorButton {}
    impl PreferencesRowImpl for MonitorButton {}
    impl ActionRowImpl for MonitorButton {}
}

glib::wrapper! {
    pub struct MonitorButton(ObjectSubclass<imp::MonitorButton>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonitorButton {
    pub fn new(monitor: &Monitor) -> Self {
        glib::Object::builder()
            .property("monitor-name", &monitor.name)
            .property("monitor-description", &monitor.description)
            .build()
    }
}
