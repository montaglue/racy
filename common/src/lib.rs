use std::{
    env, error::Error, fs::{File, OpenOptions}, io::{self, Cursor, Read}, path::PathBuf
};

const FILE_NAME: &'static str = "racy_output.bin";

#[derive(Debug)]
pub struct Event {
    pub id: u64,
    pub duration: u64,
    pub timestamp: u128,
    pub name: String,
}

pub fn serialize_events(events: &[Event]) -> Vec<u8> {
    let mut size = 0;
    for event in events.iter() {
        size += 28 + event.name.len();
    }
    let mut result = Vec::with_capacity(size);

    for event in events.iter() {
        result.extend_from_slice(&event.id.to_be_bytes());
        result.extend_from_slice(&event.timestamp.to_be_bytes());
        result.extend_from_slice(&event.duration.to_be_bytes());
        result.extend_from_slice(&(event.name.len() as u32).to_be_bytes());
        result.extend_from_slice(event.name.as_bytes());
    }
    result
}

pub fn deserialize_events(data: &[u8]) -> io::Result<Vec<Event>> {
    let mut cursor = Cursor::new(data);
    let mut events = Vec::new();

    while cursor.position() < data.len() as u64 {
        let mut buffer = [0u8; 8];
        let mut big_buffer = [0u8; 16];

        // Read id
        cursor.read_exact(&mut buffer)?;
        let id = u64::from_be_bytes(buffer);

        // Read timestamp
        cursor.read_exact(&mut big_buffer)?;
        let timestamp = u128::from_be_bytes(big_buffer);

        // Read duration
        cursor.read_exact(&mut buffer)?;
        let duration = u64::from_be_bytes(buffer);

        // Read name length
        let mut len_buffer = [0u8; 4];
        cursor.read_exact(&mut len_buffer)?;
        let name_len = u32::from_be_bytes(len_buffer) as usize;

        // Read name
        let mut name_buffer = vec![0u8; name_len];
        cursor.read_exact(&mut name_buffer)?;
        let name = String::from_utf8(name_buffer).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid UTF-8 in event name: {}", e),
            )
        })?;

        events.push(Event {
            id,
            timestamp,
            duration,
            name,
        });
    }

    Ok(events)
}

pub fn default_save_filename() -> PathBuf {
    env::temp_dir().join(FILE_NAME)
}

pub fn default_save_file() -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(default_save_filename())
}

pub fn read_events(file: PathBuf) -> Result<Vec<Event>, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(file)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    Ok(deserialize_events(&data)?)
}