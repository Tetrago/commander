use gtk::glib;
use pipewire::{context::Context, main_loop::MainLoop};
use std::{collections::HashMap, sync::mpsc, thread};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "AudioDeviceType")]
pub enum AudioDeviceType {
    #[default]
    Sink,
    Source,
}

#[derive(Debug)]
pub struct AudioDevice {
    name: String,
    device_type: AudioDeviceType,
    is_default: bool,
}

#[derive(Debug)]
enum AudioMessage {
    DeviceAdded(u32, AudioDevice),
    DeviceRemoved(u32),
}

#[derive(Debug)]
struct Terminate;

pub struct Audio {
    thread: Option<thread::JoinHandle<()>>,
    receiver: mpsc::Receiver<AudioMessage>,
    sender: pipewire::channel::Sender<Terminate>,
    devices: HashMap<u32, AudioDevice>,
}

impl Audio {
    pub fn new() -> Self {
        let (main_sender, main_receiver) = mpsc::channel();
        let (pw_sender, pw_receiver) = pipewire::channel::channel();

        let thread = Some(thread::spawn(move || {
            Self::thread(main_sender, pw_receiver)
        }));

        Self {
            thread,
            receiver: main_receiver,
            sender: pw_sender,
            devices: HashMap::new(),
        }
    }

    pub fn process_events(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                AudioMessage::DeviceAdded(id, device) => _ = self.devices.insert(id, device),
                AudioMessage::DeviceRemoved(id) => _ = self.devices.remove(&id),
            }
        }
    }

    pub fn get_devices(&self) -> Vec<&AudioDevice> {
        self.devices.values().collect()
    }

    fn thread(
        sender: mpsc::Sender<AudioMessage>,
        receiver: pipewire::channel::Receiver<Terminate>,
    ) {
        let main_loop = MainLoop::new(None).expect("Failed to create main loop");
        let context = Context::new(&main_loop).expect("Failed to create context");
        let core = context.connect(None).expect("Failed to create core");
        let registry = core.get_registry().expect("Failed to get registry");

        let _ = registry
            .add_listener_local()
            .global({
                let sender = sender.clone();
                move |global| {
                    println!("{:?}", global);

                    if global.type_ == pipewire::types::ObjectType::Node {
                        if let Some(props) = &global.props {
                            let media_class = props.get("media.class").unwrap_or("");
                            let is_input =
                                media_class.contains("Input") || media_class.contains("Source");
                            let is_output =
                                media_class.contains("Output") || media_class.contains("Sink");

                            if is_input || is_output {
                                let name = props.get("node.name").unwrap_or("Unknown").to_string();
                                let device_type = if is_input {
                                    AudioDeviceType::Source
                                } else {
                                    AudioDeviceType::Sink
                                };
                                let is_default = props.get("node.default").unwrap_or("") == "true";

                                sender
                                    .send(AudioMessage::DeviceAdded(
                                        global.id,
                                        AudioDevice {
                                            name,
                                            device_type,
                                            is_default,
                                        },
                                    ))
                                    .unwrap();
                            }
                        }
                    }
                }
            })
            .global_remove({
                let sender = sender.clone();
                move |id| {
                    sender.send(AudioMessage::DeviceRemoved(id)).unwrap();
                }
            })
            .register();

        let _ = receiver.attach(main_loop.loop_(), {
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
