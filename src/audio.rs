use gtk::glib;
use pipewire::{context::Context, main_loop::MainLoop, metadata, types::ObjectType};
use std::{
    cell::Cell,
    collections::HashMap,
    sync::{Arc, Barrier, mpsc},
    thread,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "AudioDeviceType")]
pub enum AudioDeviceType {
    #[default]
    Sink,
    Source,
}

#[derive(Debug, Eq)]
pub struct AudioDevice {
    id: u32,
    name: String,
    device_type: AudioDeviceType,
}

impl AudioDevice {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn device_type(&self) -> AudioDeviceType {
        self.device_type
    }
}

impl PartialEq for AudioDevice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for AudioDevice {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug)]
enum Message {
    DeviceAdded(AudioDevice),
    DeviceRemoved(u32),
    DefaultDeviceChanged(AudioDeviceType, u32),
}

#[derive(Debug)]
struct Terminate;

pub struct Audio {
    thread: Option<thread::JoinHandle<()>>,
    receiver: mpsc::Receiver<Message>,
    sender: pipewire::channel::Sender<Terminate>,
    devices: HashMap<u32, AudioDevice>,
    defaults: HashMap<AudioDeviceType, u32>,
}

impl Audio {
    pub fn new() -> Self {
        let (main_sender, main_receiver) = mpsc::channel();
        let (pw_sender, pw_receiver) = pipewire::channel::channel();

        let barrier = Arc::new(Barrier::new(2));

        let thread = Some(thread::spawn({
            let barrier = barrier.clone();
            move || Self::thread(main_sender, pw_receiver, barrier)
        }));

        let mut audio = Self {
            thread,
            receiver: main_receiver,
            sender: pw_sender,
            devices: HashMap::new(),
            defaults: HashMap::new(),
        };

        barrier.wait();
        audio.process_events();
        audio
    }

    pub fn process_events(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                Message::DeviceAdded(device) => _ = self.devices.insert(device.id, device),
                Message::DeviceRemoved(id) => _ = self.devices.remove(&id),
                Message::DefaultDeviceChanged(device_type, id) => {
                    println!("Changed!");
                    _ = self.defaults.insert(device_type, id)
                }
            }
        }
    }

    pub fn get_devices(&self) -> Vec<&AudioDevice> {
        self.devices.values().collect()
    }

    pub fn get_default_device(&self, device_type: AudioDeviceType) -> Option<&AudioDevice> {
        self.defaults
            .get(&device_type)
            .map(|id| self.devices.get(id).unwrap())
    }

    fn thread(
        sender: mpsc::Sender<Message>,
        receiver: pipewire::channel::Receiver<Terminate>,
        barrier: Arc<Barrier>,
    ) {
        let main_loop = MainLoop::new(None).expect("Failed to create main loop");
        let context = Context::new(&main_loop).expect("Failed to create context");
        let core = context.connect(None).expect("Failed to create core");
        let registry = core.get_registry().expect("Failed to get registry");

        let _sync = core
            .add_listener_local()
            .done({
                let sync = core.sync(0).expect("Sync failed");

                move |id, seq| {
                    if id == pipewire::core::PW_ID_CORE && seq == sync {
                        barrier.wait();
                    }
                }
            })
            .register();

        let _listener = registry
            .add_listener_local()
            .global({
                let registry = core.get_registry().unwrap();
                let sender = sender.clone();
                let metadata_listener: Cell<Option<metadata::MetadataListener>> = Cell::new(None);

                move |global| {
                    if let Some(props) = global.props {
                        if global.type_ == ObjectType::Node {
                            if let Some(media_class) = props.get("media.class") {
                                if let Some(device_type) = if media_class.contains("Source") {
                                    Some(AudioDeviceType::Source)
                                } else if media_class.contains("Sink") {
                                    Some(AudioDeviceType::Sink)
                                } else {
                                    None
                                } {
                                    let name = props
                                        .get("node.description")
                                        .unwrap_or("Unknown")
                                        .to_string();

                                    sender
                                        .send(Message::DeviceAdded(AudioDevice {
                                            id: global.id,
                                            name,
                                            device_type,
                                        }))
                                        .unwrap();
                                }
                            }
                        } else if global.type_ == ObjectType::Metadata {
                            if props
                                .get("metadata.name")
                                .map_or(true, |name| name != "default")
                            {
                                return;
                            }

                            if let Ok(metadata) = registry.bind::<metadata::Metadata, _>(global) {
                                metadata_listener.set(Some(
                                    metadata
                                        .add_listener_local()
                                        .property(move |_, key, _, value| {
                                            println!("{:?} {:?}", key, value);
                                            0
                                        })
                                        .register(),
                                ));
                            }
                        }
                    }
                }
            })
            .global_remove({
                let sender = sender.clone();
                move |id| {
                    sender.send(Message::DeviceRemoved(id)).unwrap();
                }
            })
            .register();

        let _receiver = receiver.attach(main_loop.loop_(), {
            let main_loop = main_loop.clone();
            move |_| main_loop.quit()
        });

        main_loop.run();
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.sender.send(Terminate).unwrap();

        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
