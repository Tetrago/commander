use crate::monitors::DisplaySettings;
use crate::monitors::Monitor;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::DisplaySidebarButton)]
    #[template(file = "ui/monitors/display_sidebar_button.blp")]
    pub struct DisplaySidebarButton {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[property(name = "monitor-name", get, type = String, member = name, construct_only)]
        pub monitor: RefCell<Monitor>,

        pub display_settings: RefCell<Option<DisplaySettings>>,
    }

    #[gtk::template_callbacks]
    impl DisplaySidebarButton {
        #[template_callback]
        fn on_activated(&self) {
            if let (Some(settings), monitor) = (
                self.display_settings.borrow().as_ref(),
                self.monitor.borrow(),
            ) {
                settings.set_monitor(&*monitor);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DisplaySidebarButton {
        const NAME: &'static str = "DisplaySidebarButton";
        type Type = super::DisplaySidebarButton;
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
    impl ObjectImpl for DisplaySidebarButton {}

    impl WidgetImpl for DisplaySidebarButton {}
    impl ListBoxRowImpl for DisplaySidebarButton {}
    impl PreferencesRowImpl for DisplaySidebarButton {}
    impl ActionRowImpl for DisplaySidebarButton {}
}

glib::wrapper! {
    pub struct DisplaySidebarButton(ObjectSubclass<imp::DisplaySidebarButton>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DisplaySidebarButton {
    pub fn new(monitor: &Monitor, settings: &DisplaySettings) -> Self {
        let obj: Self = glib::Object::builder()
            .property("monitor-name", &monitor.name)
            .build();

        obj.imp().monitor.replace(monitor.clone());
        obj.imp().display_settings.replace(Some(settings.clone()));
        obj
    }
}
