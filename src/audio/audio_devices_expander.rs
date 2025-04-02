use crate::audio;
use crate::audio::Audio;
use crate::audio::AudioDeviceRow;
use crate::audio::Device as AudioDevice;
use crate::audio::DeviceType as AudioDeviceType;
use crate::audio::Event;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use std::cell::Cell;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::AudioDevicesExpander)]
    #[template(file = "ui/audio/audio_devices_expander.blp")]
    pub struct AudioDevicesExpander {
        #[property(get, construct_only, builder(AudioDeviceType::default()))]
        pub device_type: Cell<AudioDeviceType>,

        #[template_child]
        pub device_icon: TemplateChild<gtk::Image>,

        pub audio: RefCell<Option<Rc<RefCell<Audio>>>>,
        pub listener: Cell<Option<audio::Listener>>,
        pub rows: RefCell<Vec<AudioDeviceRow>>,
        pub devices: RefCell<Vec<AudioDevice>>,
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

            let audio = Audio::new();

            let group = gtk::CheckButton::new();

            self.listener.replace(Some(audio.borrow_mut().add_listener({
                let audio = audio.clone();
                let device_type = self.device_type.get().clone();
                let obj = self.obj().clone();
                let default: RefCell<Option<u32>> = RefCell::default();

                move |msg| {
                    if match msg {
                        Event::DeviceAdded(device) if device.device_type == device_type => {
                            obj.imp().devices.borrow_mut().push(device.clone());

                            true
                        }
                        Event::DeviceRemoved(device) if device.device_type == device_type => {
                            obj.imp().devices.borrow_mut().retain(|elem| elem != device);
                            true
                        }
                        Event::DefaultDeviceChanged(device)
                            if device.device_type == device_type =>
                        {
                            default.replace(Some(device.id));
                            true
                        }
                        _ => false,
                    } {
                        obj.imp()
                            .rows
                            .borrow()
                            .iter()
                            .for_each(|elem| obj.remove(elem));

                        obj.imp()
                            .devices
                            .borrow_mut()
                            .sort_by(|a, b| a.description.cmp(&b.description));

                        obj.imp().rows.replace(
                            obj.imp()
                                .devices
                                .borrow()
                                .iter()
                                .map(|device| {
                                    let row =
                                        AudioDeviceRow::new(audio.clone(), device, Some(&group));

                                    if default.borrow().map_or(false, |id| id == device.id) {
                                        row.imp().check_button.set_active(true);
                                        default.replace(None);
                                    }

                                    obj.add_row(&row);
                                    row
                                })
                                .collect(),
                        );
                    }
                }
            })));

            self.audio.replace(Some(audio));
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
