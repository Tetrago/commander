use crate::audio::AudioDeviceType;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use std::cell::Cell;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::AudioDevicesExpander)]
    #[template(resource = "/io/github/tetrago/commander/audio_devices_expander.ui")]
    pub struct AudioDevicesExpander {
        #[property(get, construct_only, builder(AudioDeviceType::default()))]
        pub device_type: Cell<AudioDeviceType>,

        #[template_child]
        pub device_icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioDevicesExpander {
        const NAME: &'static str = "AudioDevicesExpander";
        type Type = super::AudioDevicesExpander;
        type ParentType = adw::ExpanderRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AudioDevicesExpander {
        fn constructed(&self) {
            self.parent_constructed();

            let (title, icon) = match self.device_type.get() {
                AudioDeviceType::Sink => ("Sink", "audio-speakers-symbolic"),
                AudioDeviceType::Source => ("Source", "audio-input-microphone-symbolic"),
            };

            self.obj().set_title(title);
            self.device_icon.set_icon_name(Some(icon));
        }
    }

    impl WidgetImpl for AudioDevicesExpander {}
    impl ListBoxRowImpl for AudioDevicesExpander {}
    impl PreferencesRowImpl for AudioDevicesExpander {}
    impl ExpanderRowImpl for AudioDevicesExpander {}
}

glib::wrapper! {
    pub struct AudioDevicesExpander(ObjectSubclass<imp::AudioDevicesExpander>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ExpanderRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
