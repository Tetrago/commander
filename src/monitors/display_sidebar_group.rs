use crate::monitors;
use crate::monitors::DisplaySettings;
use crate::monitors::DisplaySidebarButton;
use crate::monitors::Monitors;
use adw::prelude::PreferencesGroupExt;
use gtk::glib;

mod imp {
    use super::*;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use glib::subclass::Signal;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::OnceLock;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::DisplaySidebarGroup)]
    #[template(file = "ui/monitors/display_sidebar_group.blp")]
    pub struct DisplaySidebarGroup {
        #[property(get, set)]
        pub display_settings: RefCell<Option<DisplaySettings>>,

        pub monitors: RefCell<Option<Rc<RefCell<Monitors>>>>,
        pub listener: RefCell<Option<monitors::Listener>>,
        pub buttons: RefCell<Vec<DisplaySidebarButton>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DisplaySidebarGroup {
        const NAME: &'static str = "DisplaySidebarGroup";
        type Type = super::DisplaySidebarGroup;
        type ParentType = adw::PreferencesGroup;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DisplaySidebarGroup {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("activated").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_display_settings_notify(|obj| {
                if let Some(settings) = obj.imp().display_settings.borrow().as_ref() {
                    let monitors = Monitors::new();

                    obj.imp()
                        .listener
                        .replace(Some(monitors.borrow().add_listener({
                            let obj = obj.clone();
                            let settings = settings.clone();

                            move |event| match event {
                                monitors::Event::MonitorAdded(monitor) => {
                                    let btn = DisplaySidebarButton::new(&monitor, &settings);
                                    obj.add(&btn);

                                    btn.connect_activated({
                                        let obj = obj.clone();
                                        move |_| obj.emit_by_name("activated", &[])
                                    });

                                    obj.imp().buttons.borrow_mut().push(btn)
                                }
                                monitors::Event::MonitorRemoved(monitor) => {
                                    let mut buttons = obj.imp().buttons.borrow_mut();

                                    for i in 0..buttons.len() {
                                        if *buttons[i].imp().monitor.borrow() == *monitor {
                                            buttons.remove(i);
                                            break;
                                        }
                                    }
                                }
                            }
                        })));

                    obj.imp().monitors.replace(Some(monitors));
                }
            });
        }
    }

    impl WidgetImpl for DisplaySidebarGroup {}
    impl PreferencesGroupImpl for DisplaySidebarGroup {}
}

glib::wrapper! {
    pub struct DisplaySidebarGroup(ObjectSubclass<imp::DisplaySidebarGroup>)
        @extends adw::PreferencesGroup, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
