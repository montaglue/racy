use std::time::SystemTime;

use common::{default_save_filename, read_events, Event};

use crate::event::{Events, EventsBuilder};

pub fn load_from_file() -> Events {
    let raw_events = read_events(default_save_filename()).unwrap();

    let mut builder =  EventsBuilder::new();
    builder.add_vec(raw_events);
    builder.build()
}

pub fn example() -> Events {
    let process_id = 12345;  // Single process ID for all events
    let base_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let events= vec![
        Event {
            id: process_id,
            duration: 150_000_000,  // 150ms in microseconds
            timestamp: base_timestamp,
            name: "database_query".to_string(),
        },
        Event {
            id: process_id,
            duration: 45_000_000,   // 45ms
            timestamp: base_timestamp + 200_000_000,  // 200ms later
            name: "user_authentication".to_string(),
        },
        Event {
            id: process_id,
            duration: 2_500_000_000, // 2.5s
            timestamp: base_timestamp + 500_000_000,  // 500ms later
            name: "file_processing".to_string(),
        },
        Event {
            id: process_id,
            duration: 75_000_000,   // 75ms
            timestamp: base_timestamp + 800_000_000,
            name: "api_request".to_string(),
        },
        Event {
            id: process_id,
            duration: 1_200_000_000, // 1.2s
            timestamp: base_timestamp + 1_000_000_000, // 1s later
            name: "image_compression".to_string(),
        },
        Event {
            id: process_id,
            duration: 25_000_000,   // 25ms
            timestamp: base_timestamp + 1_300_000_000,
            name: "cache_lookup".to_string(),
        },
        Event {
            id: process_id,
            duration: 500_000_000,  // 500ms
            timestamp: base_timestamp + 1_500_000_000,
            name: "network_request".to_string(),
        },
        Event {
            id: process_id,
            duration: 90_000_000,   // 90ms
            timestamp: base_timestamp + 2_000_000_000, // 2s later
            name: "json_parsing".to_string(),
        },
        Event {
            id: process_id,
            duration: 3_000_000_000, // 3s
            timestamp: base_timestamp + 2_200_000_000,
            name: "video_transcoding".to_string(),
        },
        Event {
            id: process_id,
            duration: 15_000_000,   // 15ms
            timestamp: base_timestamp + 2_500_000_000,
            name: "memory_allocation".to_string(),
        },
        Event {
            id: process_id,
            duration: 800_000_000,  // 800ms
            timestamp: base_timestamp + 3_000_000_000, // 3s later
            name: "encryption".to_string(),
        },
        Event {
            id: process_id,
            duration: 120_000_000,  // 120ms
            timestamp: base_timestamp + 3_500_000_000,
            name: "template_rendering".to_string(),
        },
        Event {
            id: process_id,
            duration: 65_000_000,   // 65ms
            timestamp: base_timestamp + 4_000_000_000, // 4s later
            name: "validation".to_string(),
        },
        Event {
            id: process_id,
            duration: 1_800_000_000, // 1.8s
            timestamp: base_timestamp + 4_200_000_000,
            name: "data_synchronization".to_string(),
        },
        Event {
            id: process_id,
            duration: 35_000_000,   // 35ms
            timestamp: base_timestamp + 5_000_000_000, // 5s later
            name: "logging".to_string(),
        },
    ];

    let mut builder = EventsBuilder::new();
    builder.add_vec(events);

    builder.build()
}
