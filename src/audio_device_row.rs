use crate::audio::Audio;
use crate::audio::Device as AudioDevice;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::{cell::RefCell, rc::Rc};

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::AudioDeviceRow)]
    #[template(file = "ui/audio_device_row.blp")]
    pub struct AudioDeviceRow {
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,

        #[property(name = "device-id", get, type = u32, member = id, construct_only)]
        #[property(name = "device-name", get, type = String, member = description, construct_only)]
        pub device: RefCell<AudioDevice>,

        pub audio: RefCell<Option<Rc<RefCell<Audio>>>>,
    }

    #[gtk::template_callbacks]
    impl AudioDeviceRow {
        #[template_callback]
        fn on_toggled(&self) {
            if self.check_button.is_active() {
                if let Some(audio) = self.audio.borrow().as_ref() {
                    if let Ok(audio) = audio.try_borrow() {
                        audio.set_default_device(&*self.device.borrow());
                    }
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioDeviceRow {
        const NAME: &'static str = "AudioDeviceRow";
        type Type = super::AudioDeviceRow;
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

impl AudioDeviceRow {
    pub fn new(
        audio: Rc<RefCell<Audio>>,
        device: &AudioDevice,
        group: Option<&gtk::CheckButton>,
    ) -> Self {
        let obj: Self = glib::Object::builder()
            .property("device-id", device.id)
            .property("device-name", &device.description)
            .build();

        obj.imp().audio.replace(Some(audio));
        obj.imp().device.replace(device.clone());

        obj.imp().check_button.set_group(group);
        obj
    }
}
