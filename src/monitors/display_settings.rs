use crate::monitors::Monitor;
use crate::monitors::Monitors;
use adw::prelude::*;
use adw::subclass::prelude::PreferencesPageImpl;
use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::sync::OnceLock;

fn run_scripts(name: &str) {
    static PATH: OnceLock<String> = OnceLock::new();
    let path = PATH.get_or_init(|| {
        std::env::var("XDG_DATA_HOME")
            .map(|var| format!("{}/commander/monitor.d", var))
            .expect("Could not determine scripts directory")
    });

    if let Ok(dir) = std::fs::read_dir(path) {
        for entry in dir {
            if let Ok(entry) = entry {
                let path = entry.path();
                let _ = std::process::Command::new(path).arg(name).status();
            }
        }
    }
}

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::DisplaySettings)]
    #[template(file = "ui/monitors/display_settings.blp")]
    pub struct DisplaySettings {
        #[property(name = "monitor-name", get, type = String, member = name)]
        #[property(name = "monitor-description", get, type = String, member = description)]
        #[property(name = "monitor-make", get, type = String, member = make)]
        #[property(name = "monitor-model", get, type = String, member = model)]
        #[property(name = "monitor-serial", get, type = String, member = serial)]
        #[property(name = "monitor-scale", get = Self::get_scale, set = Self::set_scale, type = u32)]
        #[property(name = "monitor-transform", get, set, type = u64, member = transform)]
        #[property(name = "monitor-x", get = Self::get_x, set = Self::set_x, type = String)]
        #[property(name = "monitor-y", get = Self::get_y, set = Self::set_y, type = String)]
        #[property(name = "monitor-available-modes", get = Self::get_available_modes, type = gtk::StringList)]
        #[property(name = "monitor-mode", get = Self::get_mode, type = u32)]
        #[property(name = "monitor-disabled", get, set, type = bool, member = disabled)]
        pub monitor: RefCell<Monitor>,

        #[property(get)]
        pub modified: Cell<bool>,

        #[property(set = Self::set_selected_mode, type = u32)]
        pub selected_mode: RefCell<String>,

        #[template_child]
        pub position_x: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub position_y: TemplateChild<adw::EntryRow>,

        pub state: RefCell<Monitor>,
    }

    #[gtk::template_callbacks]
    impl DisplaySettings {
        fn get_scale(&self) -> u32 {
            (self.monitor.borrow().scale * 100.0) as u32
        }

        fn set_scale(&self, value: u32) {
            self.monitor.borrow_mut().scale = (value as f64) / 100.0;
        }

        fn get_available_modes(&self) -> gtk::StringList {
            self.monitor
                .borrow()
                .available_modes
                .iter()
                .cloned()
                .collect()
        }

        fn get_x(&self) -> String {
            self.monitor.borrow().x.to_string()
        }

        fn set_x(&self, value: String) {
            if let Ok(value) = value.parse::<i64>() {
                self.monitor.borrow_mut().x = value;
            }
        }

        fn get_y(&self) -> String {
            self.monitor.borrow().y.to_string()
        }

        fn set_y(&self, value: String) {
            if let Ok(value) = value.parse::<i64>() {
                self.monitor.borrow_mut().y = value;
            }
        }

        fn get_mode(&self) -> u32 {
            let monitor = self.monitor.borrow();

            let resolution = format!("{}x{}", monitor.width, monitor.height);
            let mut candidate: Option<(u32, f64)> = None;

            // The refresh rate given by Hyprland might not appear in the monitor's available
            // modes, so we need to handle that by picking the closest option
            for i in 0..monitor.available_modes.len() {
                let mode = &monitor.available_modes[i];

                if !mode.starts_with(&resolution) {
                    continue;
                }

                if let Some(refresh_rate) = (|| {
                    let (_, refresh_rate) = mode.split_once('@')?;
                    refresh_rate[..refresh_rate.len() - 2].parse::<f64>().ok()
                })() {
                    let difference = (refresh_rate - monitor.refresh_rate).abs();

                    if let Some((_, delta)) = candidate {
                        if delta > difference {
                            candidate.replace((i as u32, difference));
                        }
                    } else {
                        candidate.replace((i as u32, difference));
                    }
                }
            }

            candidate.map(|elem| elem.0).unwrap_or(0)
        }

        fn set_selected_mode(&self, value: u32) {
            if let Some(mode) = self.monitor.borrow().available_modes.get(value as usize) {
                self.selected_mode.replace(mode.clone());
            }
        }

        fn update_mode(&self) {
            let value = self.selected_mode.borrow();

            if let Some((width, height, refresh_rate)) = (|| {
                let (resolution, refresh_rate) = value.split_once('@')?;

                let (width, height) = resolution.split_once('x')?;
                let width = width.parse::<u64>().ok()?;
                let height = height.parse::<u64>().ok()?;

                let refresh_rate = refresh_rate[..refresh_rate.len() - 2].parse::<f64>().ok()?;

                Some((width, height, refresh_rate))
            })() {
                let mut monitor = self.monitor.borrow_mut();
                monitor.width = width;
                monitor.height = height;
                monitor.refresh_rate = refresh_rate;
            }
        }

        #[template_callback]
        fn on_change_x(&self) {
            self.obj().set_monitor_x(self.position_x.text().to_string());
        }

        #[template_callback]
        fn on_change_y(&self) {
            self.obj().set_monitor_y(self.position_y.text().to_string());
        }

        #[template_callback]
        fn reset(&self) {
            let monitor = self.state.borrow().clone();
            self.obj().set_monitor(&monitor);
        }

        #[template_callback]
        fn apply(&self) {
            self.update_mode();
            let monitor = self.monitor.borrow().clone();

            let command = if monitor.disabled {
                format!("keyword monitor {},disable\n", monitor.name)
            } else {
                format!(
                    "keyword monitor {},{}x{}@{},{}x{},{},transform,{}\n",
                    monitor.name,
                    monitor.width,
                    monitor.height,
                    monitor.refresh_rate,
                    monitor.x,
                    monitor.y,
                    monitor.scale,
                    monitor.transform
                )
            };

            self.obj().set_monitor(&monitor);
            Monitors::issue(command.as_bytes()).unwrap();
            run_scripts(&monitor.name);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DisplaySettings {
        const NAME: &'static str = "DisplaySettings";
        type Type = super::DisplaySettings;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DisplaySettings {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);

            if !self.modified.get() {
                self.modified.replace(true);
                self.obj().notify_modified();
            }
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for DisplaySettings {}
    impl PreferencesPageImpl for DisplaySettings {}
}

glib::wrapper! {
    pub struct DisplaySettings(ObjectSubclass<imp::DisplaySettings>)
        @extends adw::PreferencesPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DisplaySettings {
    pub fn set_monitor(&self, monitor: &Monitor) {
        self.imp().state.replace(monitor.clone());
        self.imp().monitor.replace(monitor.clone());

        self.notify_monitor_name();
        self.notify_monitor_description();
        self.notify_monitor_make();
        self.notify_monitor_model();
        self.notify_monitor_serial();
        self.notify_monitor_scale();
        self.notify_monitor_transform();
        self.notify_monitor_x();
        self.notify_monitor_y();
        self.notify_monitor_available_modes();
        self.notify_monitor_mode();
        self.notify_monitor_disabled();

        self.imp().modified.replace(false);
        self.notify_modified();
    }
}
