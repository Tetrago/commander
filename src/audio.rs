use gtk::glib;
use pipewire::{
    context::Context,
    main_loop::MainLoop,
    metadata::{Metadata, MetadataListener},
    types::ObjectType,
};
use serde_json::json;
use std::{
    cell::Cell,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    thread,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "DeviceType")]
pub enum DeviceType {
    #[default]
    Sink,
    Source,
}

#[derive(Clone, Debug, Default, Eq)]
pub struct Device {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub device_type: DeviceType,
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub enum Event {
    DeviceAdded(Device),
    DeviceRemoved(Device),
    DefaultDeviceChanged(Device),
}

enum Message {
    DeviceAdded(Device),
    DeviceRemoved(u32),
    DefaultDeviceChanged(DeviceType, String),
}

#[derive(Debug)]
enum Command {
    MakeDefault(Device),
    Terminate,
}

pub struct Listener(
    Weak<RefCell<Vec<Rc<dyn Fn(&Event)>>>>,
    Weak<dyn Fn(&Event) + 'static>,
);

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(listeners) = self.0.upgrade() {
            listeners
                .borrow_mut()
                .retain(|elem| !self.1.ptr_eq(&Rc::downgrade(elem)));
        }
    }
}

pub struct Audio {
    thread: Option<thread::JoinHandle<()>>,
    sender: pipewire::channel::Sender<Command>,
    listeners: Rc<RefCell<Vec<Rc<dyn Fn(&Event)>>>>,
    devices: HashMap<u32, Device>,
    defaults: HashMap<DeviceType, u32>,
}

impl Audio {
    pub fn new() -> Rc<RefCell<Self>> {
        let (tx, rx) = async_channel::unbounded::<Message>();
        let (sender, receiver) = pipewire::channel::channel();

        let thread = Some(thread::spawn(move || Self::thread(tx, receiver)));

        let audio = Rc::new(RefCell::new(Self {
            thread,
            sender,
            listeners: Rc::default(),
            devices: HashMap::new(),
            defaults: HashMap::new(),
        }));

        glib::timeout_add_local(std::time::Duration::from_millis(100), {
            let audio = Rc::downgrade(&audio);

            move || {
                if let Some(audio) = audio.upgrade() {
                    let mut audio = audio.borrow_mut();

                    while let Ok(msg) = rx.try_recv() {
                        if let Some(event) = match msg {
                            Message::DeviceAdded(ref device) => {
                                let device = device.clone();
                                _ = audio.devices.insert(device.id, device.clone());
                                Some(Event::DeviceAdded(device))
                            }
                            Message::DeviceRemoved(id) => audio
                                .devices
                                .remove(&id)
                                .map(|device| Event::DeviceRemoved(device)),
                            Message::DefaultDeviceChanged(device_type, ref name) => audio
                                .devices
                                .iter()
                                .filter_map(|(_, device)| {
                                    if device.name == *name {
                                        Some(device.clone())
                                    } else {
                                        None
                                    }
                                })
                                .next()
                                .map(|device| {
                                    audio.defaults.insert(device_type, device.id);
                                    Event::DefaultDeviceChanged(device)
                                }),
                        } {
                            audio.listeners.borrow().iter().for_each(|f| f(&event));
                        }
                    }

                    gtk::glib::ControlFlow::Continue
                } else {
                    gtk::glib::ControlFlow::Break
                }
            }
        });

        audio
    }

    pub fn set_default_device(&self, device: &Device) {
        self.sender
            .send(Command::MakeDefault(device.clone()))
            .unwrap();
    }

    pub fn add_listener<T>(&self, listener: T) -> Listener
    where
        T: Fn(&Event) + 'static,
    {
        self.devices
            .iter()
            .for_each(|(_, device)| listener(&Event::DeviceAdded(device.clone())));

        self.defaults
            .iter()
            .map(|(device_type, id)| (device_type, self.devices.get(id).unwrap()))
            .for_each(|(_, device)| listener(&Event::DefaultDeviceChanged(device.clone())));

        let listener = Rc::new(listener);
        self.listeners.borrow_mut().push(listener.clone());

        Listener(
            Rc::downgrade(&self.listeners),
            Rc::downgrade(&listener) as Weak<dyn Fn(&Event) + 'static>,
        )
    }

    fn thread(
        sender: async_channel::Sender<Message>,
        receiver: pipewire::channel::Receiver<Command>,
    ) {
        let main_loop = MainLoop::new(None).expect("Failed to create main loop");
        let context = Context::new(&main_loop).expect("Failed to create context");
        let core = context.connect(None).expect("Failed to create core");
        let registry = Rc::new(core.get_registry().expect("Failed to get registry"));
        let metadata: Rc<RefCell<Option<Metadata>>> = Rc::default();

        let _listener = registry
            .add_listener_local()
            .global({
                let sender = sender.clone();
                let registry = registry.clone();

                let metadata = metadata.clone();
                let metadata_listener: Cell<Option<MetadataListener>> = Cell::default();

                move |global| {
                    if let Some(props) = global.props {
                        if global.type_ == ObjectType::Node {
                            if let Some(media_class) = props.get("media.class") {
                                if let Some(device_type) = if media_class.contains("Source") {
                                    Some(DeviceType::Source)
                                } else if media_class.contains("Sink") {
                                    Some(DeviceType::Sink)
                                } else {
                                    None
                                } {
                                    let name =
                                        props.get("node.name").unwrap_or("Unknown").to_string();

                                    let description = props
                                        .get("node.description")
                                        .unwrap_or("Unknown")
                                        .to_string();

                                    sender
                                        .try_send(Message::DeviceAdded(Device {
                                            id: global.id,
                                            name,
                                            description,
                                            device_type,
                                        }))
                                        .unwrap();
                                }
                            }
                        } else if global.type_ == ObjectType::Metadata {
                            if props.get("metadata.name") != Some("default") {
                                return;
                            }

                            if let Ok(meta) = registry.bind::<Metadata, _>(global) {
                                metadata_listener.set(Some(
                                    meta.add_listener_local()
                                        .property({
                                            let sender = sender.clone();

                                            move |_, key, _, value| {
                                                if let Some(device_type) = match key {
                                                    Some("default.audio.sink") => {
                                                        Some(DeviceType::Sink)
                                                    }
                                                    Some("default.audio.source") => {
                                                        Some(DeviceType::Source)
                                                    }
                                                    _ => None,
                                                } {
                                                    if let Some(value) = value.map_or(None, |v| {
                                                        serde_json::from_str(v).ok()
                                                            as Option<serde_json::Value>
                                                    }) {
                                                        if let Some(name) = value["name"]
                                                            .as_str()
                                                            .map(str::to_owned)
                                                        {
                                                            sender
                                                                .try_send(
                                                                    Message::DefaultDeviceChanged(
                                                                        device_type,
                                                                        name,
                                                                    ),
                                                                )
                                                                .unwrap();
                                                        }
                                                    }
                                                }

                                                0
                                            }
                                        })
                                        .register(),
                                ));

                                metadata.replace(Some(meta));
                            }
                        }
                    }
                }
            })
            .global_remove({
                let sender = sender.clone();
                move |id| sender.try_send(Message::DeviceRemoved(id)).unwrap()
            })
            .register();

        let _receiver = receiver.attach(main_loop.loop_(), {
            let main_loop = main_loop.clone();
            let metadata = metadata.clone();

            move |command| match command {
                Command::MakeDefault(device) => {
                    if let Some(metadata) = metadata.borrow().as_ref() {
                        let data = json!({"name": device.name});

                        metadata.set_property(
                            0,
                            match device.device_type {
                                DeviceType::Sink => "default.audio.sink",
                                DeviceType::Source => "default.audio.source",
                            },
                            Some("Spa:String:JSON"),
                            Some(&data.to_string()),
                        );
                    }
                }
                Command::Terminate => main_loop.quit(),
            }
        });

        main_loop.run();
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.sender.send(Command::Terminate).unwrap();

        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
