#![feature(thread_id_value)]

use common::{Event, default_save_file, default_save_filename, serialize_events};
pub use racy_macro::profile;

use std::{
    error::Error,
    fs::OpenOptions,
    io::Write,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::SystemTime,
};

use thread_local::ThreadLocal;

static EVENTS: ThreadLocal<Mutex<Vec<Event>>> = ThreadLocal::new();
static ATEXIT_REGISTERED: AtomicBool = AtomicBool::new(false);
static SPILL_CONSTANT: usize = 100;

pub struct ScopedProfiler {
    id: u64,
    timestamp: SystemTime,
    name: String,
}

impl ScopedProfiler {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let id = thread::current().id().as_u64().into();
        let timestamp = SystemTime::now();
        Self {
            id,
            timestamp,
            name,
        }
    }
}

impl Drop for ScopedProfiler {
    fn drop(&mut self) {
        let end = SystemTime::now();
        let mut name = String::new();
        std::mem::swap(&mut name, &mut self.name);

        let event = Event {
            id: self.id,
            duration: end.duration_since(self.timestamp).unwrap().as_nanos() as u64,
            timestamp: end
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            name,
        };
        record_event(event).unwrap()
    }
}

fn record_event(event: Event) -> Result<(), Box<dyn Error + 'static>> {
    let mut events = EVENTS
        .get_or(|| Mutex::new(Vec::with_capacity(SPILL_CONSTANT)))
        .try_lock()?;

    events.push(event);
    if events.len() > SPILL_CONSTANT {
        dump_current(&mut events)?;
    }
    Ok(())
}

fn dump_current(events: &mut Vec<Event>) -> Result<(), Box<dyn Error + 'static>> {
    let mut file = default_save_file()?;
    let buffer = serialize_events(events.as_ref());
    file.write(&buffer)?;
    file.flush()?;
    events.clear();
    Ok(())
}

fn dump_completion() -> Result<(), Box<dyn Error + 'static>> {
    let mut file = default_save_file()?;
    for events in EVENTS.iter() {
        let events = events.lock()?;
        let buff = serialize_events(&events);
        file.write(&buff)?;
        file.flush()?;
    }
    Ok(())
}

extern "C" fn dump_completion_marker() {
    if let Err(err) = dump_completion() {
        eprintln!("Racy error: {err}")
    }
}

pub fn init_profiler() {
    if !ATEXIT_REGISTERED.swap(true, Ordering::SeqCst) {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(default_save_filename())
            .unwrap();
        unsafe {
            libc::atexit(dump_completion_marker);
        }
    }
}

#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _profiler = $crate::ScopedProfiler::new($name);
    };
    ($name:expr, $metadata:expr) => {
        let _profiler = ScopedProfiler::with_metadata($name, $metadata);
    };
}
