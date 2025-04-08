use gtk::glib;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::OnceLock;
use std::sync::mpsc;
use std::thread;

mod display_settings;
mod display_sidebar_button;
mod display_sidebar_group;

pub use display_settings::*;
pub use display_sidebar_button::*;
pub use display_sidebar_group::*;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Monitor {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub width: u64,
    pub height: u64,
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f64,
    pub x: i64,
    pub y: i64,
    pub scale: f64,
    pub transform: u64,
    pub disabled: bool,
    #[serde(rename = "availableModes")]
    pub available_modes: Vec<String>,
}

impl PartialEq for Monitor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Monitor {}

#[derive(Clone, Debug)]
pub enum Event {
    MonitorAdded(Monitor),
    MonitorRemoved(Monitor),
}

#[derive(Clone, Debug)]
enum Message {
    MonitorAdded(String),
    MonitorRemoved(String),
}

struct Terminate;

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

pub struct Monitors {
    sender: mpsc::Sender<Terminate>,
    thread: Option<thread::JoinHandle<()>>,
    monitors: HashMap<String, Monitor>,
    listeners: Rc<RefCell<Vec<Rc<dyn Fn(&Event)>>>>,
}

impl Monitors {
    pub fn new() -> Rc<RefCell<Self>> {
        let (ctx, crx) = mpsc::channel::<Terminate>();
        let (mtx, mrx) = mpsc::channel::<Message>();

        let handle = thread::spawn(move || Self::thread(mtx, crx));

        let monitors = Rc::new(RefCell::new(Self {
            sender: ctx,
            thread: Some(handle),
            monitors: HashMap::new(),
            listeners: Rc::default(),
        }));

        Self::get_monitors().iter().for_each({
            let mut monitors = monitors.borrow_mut();

            move |elem| {
                monitors.monitors.insert(elem.name.clone(), elem.clone());

                monitors
                    .listeners
                    .borrow()
                    .iter()
                    .for_each(|listener| listener(&Event::MonitorAdded(elem.clone())));
            }
        });

        glib::timeout_add_local(std::time::Duration::from_millis(100), {
            let monitors = Rc::downgrade(&monitors);

            move || {
                if let Some(monitors) = monitors.upgrade() {
                    let mut monitors = monitors.borrow_mut();

                    while let Ok(msg) = mrx.try_recv() {
                        if let Some(event) = match msg {
                            Message::MonitorAdded(name) => {
                                if let Some(monitor) =
                                    Self::get_monitors().iter().find(|elem| elem.name == name)
                                {
                                    monitors
                                        .monitors
                                        .insert(monitor.name.clone(), monitor.clone());
                                    Some(Event::MonitorAdded(monitor.clone()))
                                } else {
                                    None
                                }
                            }
                            Message::MonitorRemoved(name) => monitors
                                .monitors
                                .get(&name)
                                .map(|monitor| Event::MonitorRemoved(monitor.clone())),
                        } {
                            monitors
                                .listeners
                                .borrow()
                                .iter()
                                .for_each(|listener| listener(&event));
                        }
                    }

                    glib::ControlFlow::Continue
                } else {
                    glib::ControlFlow::Break
                }
            }
        });

        monitors
    }

    pub fn add_listener<T>(&self, listener: T) -> Listener
    where
        T: Fn(&Event) + 'static,
    {
        self.monitors
            .iter()
            .for_each(|monitor| listener(&Event::MonitorAdded(monitor.1.clone())));

        let listener = Rc::new(listener);
        self.listeners.borrow_mut().push(listener.clone());

        Listener(
            Rc::downgrade(&self.listeners),
            Rc::downgrade(&listener) as Weak<dyn Fn(&Event) + 'static>,
        )
    }

    pub fn get_monitors() -> Vec<Monitor> {
        let raw = Self::issue(b"j/monitors\n").unwrap();
        serde_json::from_str(&raw).expect("Invalid response from Hyprland")
    }

    pub fn issue(command: &[u8]) -> Result<String, std::io::Error> {
        let mut sock = Self::connect(".socket.sock");
        sock.write(command)?;

        let mut raw = String::new();
        sock.read_to_string(&mut raw)?;

        Ok(raw)
    }

    fn connect(name: &str) -> UnixStream {
        static PATH: OnceLock<String> = OnceLock::new();
        let path = PATH.get_or_init(|| {
            let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("Could not find XDG_RUNTIME_DIR");
            let instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
                .expect("Could not find HYPRLAND_INSTANCE_SIGNATURE");

            format!("{}/hypr/{}", runtime_dir, instance_sig)
        });

        UnixStream::connect(format!("{}/{}", path, name))
            .expect(&format!("Could not connect to {}", name))
    }

    fn thread(sender: mpsc::Sender<Message>, receiver: mpsc::Receiver<Terminate>) {
        let mut event_sock = Self::connect(".socket2.sock");
        event_sock.set_nonblocking(true).unwrap();

        let mut buffer = Vec::<u8>::new();

        'outer: loop {
            if let Ok(_) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                break 'outer;
            }

            let mut buf = [0u8; 256];

            while let Ok(n) = event_sock.read(&mut buf) {
                buffer.extend_from_slice(&buf[..n]);

                while let Some(pos) = buffer.iter().position(|&elem| elem == b'\n') {
                    let line: Vec<u8> = buffer.drain(..=pos).collect();

                    if let Ok(event) = String::from_utf8(line) {
                        if let Some((event, args)) = event.split_once('>') {
                            if let Some(event) = match event {
                                "monitoradded" => Some(Message::MonitorAdded(args[1..].to_owned())),
                                "monitorremoved" => {
                                    Some(Message::MonitorRemoved(args[1..].to_owned()))
                                }
                                _ => None,
                            } {
                                sender.send(event).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Drop for Monitors {
    fn drop(&mut self) {
        self.sender.send(Terminate).unwrap();

        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
