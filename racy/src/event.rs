use std::{collections::HashMap, u128};

use common::Event;
use serde::{Deserialize, Serialize};

pub struct EventsBuilder {
    events: Vec<Event>,
}

impl EventsBuilder {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn add_vec(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }

    fn partition(events: Vec<EventSpan>) -> HashMap<u64, Thread> {
        let mut result: HashMap<u64, Thread> = HashMap::new();

        for event in events {
            let id = event.id;
            result.entry(id).or_insert(Thread::new(id)).spans.push(event);
        }

        result
    }

    fn convert(raw_events: Vec<Event>, min_timestamp: u128) -> Vec<EventSpan> {
        raw_events
            .into_iter()
            .map(|event| EventSpan {
                id: event.id,
                duration: event.duration,
                timestamp: (event.timestamp - min_timestamp) as u64,
                depth: 0,
                name: event.name,
            })
            .collect()
    }

    fn update_depth(spans: &mut Vec<EventSpan>) {
        spans.sort();
        let mut stack: Vec<u64> = Vec::new();
        for span in spans {
            while stack.last().is_some() && stack.last().unwrap() <= &span.timestamp {
                stack.pop();
            }
            span.depth = stack.len() as u64;
            stack.push(span.timestamp + span.duration);
        }
    }

    pub fn build(self) -> Events {
        let min_timestamp = self
            .events
            .iter()
            .map(|e| e.timestamp)
            .min().unwrap();

        let spans = Self::convert(self.events, min_timestamp);

        let total_duration = spans.iter().map(|event| event.timestamp + event.duration).max().unwrap();


        let mut partitioned = Self::partition(spans);

        partitioned.values_mut().for_each( |thread| {
            Self::update_depth(&mut thread.spans);
        });

        Events {
            start_time: min_timestamp,
            threads: partitioned,
            total_duration,   
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: u64,
    pub spans: Vec<EventSpan>,
}

impl Thread {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            spans: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventSpan {
    pub id: u64,
    pub duration: u64,
    pub timestamp: u64,
    pub depth: u64,
    pub name: String,
}

impl PartialOrd for EventSpan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.timestamp.partial_cmp(&other.timestamp) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        match self.duration.partial_cmp(&other.duration) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord.map(|ord| ord.reverse()),
        }

        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        
        match self.depth.partial_cmp(&other.depth) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for EventSpan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Events {
    pub start_time: u128,
    pub total_duration: u64,
    pub threads: HashMap<u64, Thread>,
}

impl Events {
    pub fn clear(&mut self) {
        self.threads.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

    pub fn thread_ids(&self) -> impl Iterator<Item = &u64> {
        self.threads.keys()
    }
}
