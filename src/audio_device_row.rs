use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use std::cell::RefCell;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::AudioDeviceRow)]
    #[template(file = "ui/audio_device_row.blp")]
    pub struct AudioDeviceRow {
        #[property(get, construct_only)]
        pub device_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioDeviceRow {
        const NAME: &'static str = "AudioDeviceRow";
        type Type = super::AudioDeviceRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl AudioDeviceRow {
        #[template_callback]
        fn toggled(&self) {}
    }

    #[glib::derived_properties]
    impl ObjectImpl for AudioDeviceRow {}

    impl WidgetImpl for AudioDeviceRow {}
    impl ListBoxRowImpl for AudioDeviceRow {}
    impl PreferencesRowImpl for AudioDeviceRow {}
    impl ActionRowImpl for AudioDeviceRow {}
}

glib::wrapper! {
    pub struct AudioDeviceRow(ObjectSubclass<imp::AudioDeviceRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
