//! A record describing a single event
//!
//! The Record contained here is essentially a strongly
//! opinionated view of what an event should looke like
//! in a basic event store.
//!
//! The entity fields `entity_id` and `entity_type` are
//! used to identify the thing that this record is about
//! - ideally with a UUID identifier and a unique type name.
//!
//! The event fields, `event_name` and `event_data` are used
//! to describe the event.  The name should be self-explanatory,
//! and the event_data is expected to be some form of text
//! serialization, such as JSON. No attempt it make to intepret
//! the data, it is treated as a UTF8 string, and stored/retrieved
//! as UTF8 bytes.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize, Serialize)]
///A single event record
pub struct Record<'a> {
    /// A unique identifier for this record
    pub id: Uuid,

    /// Number of sends since the epoch
    pub timestamp: Duration,

    /// The ID of the entity, which is essentially a link back to the
    /// Index that points to this record.
    pub entity_id: Uuid,

    /// The string representation of the entity's type
    pub entity_type: &'a str,

    /// The name of the event
    pub event_name: &'a str,

    /// The event data itself, probably in JSON.
    pub event_data: &'a str,
}

impl<'a> Record<'a> {
    /// Creates a new event record and sets the timestamp
    /// to the number of seconds since the epoch. Currently
    /// this is not set by a monotonic clock :(
    pub fn new(
        entity_id: Uuid,
        entity_type: &'a str,
        event_name: &'a str,
        event_data: &'a str,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        Record {
            id: Uuid::new_v4(),
            timestamp: ts,
            entity_id,
            entity_type,
            event_name,
            event_data,
        }
    }

    /// Converts bytes (from storage) into a record
    pub fn decode(val: &'a [u8]) -> Self {
        bincode::deserialize(val).unwrap()
    }

    /// Encode the Record as a vec of bytes for storage
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Record;

    use uuid::Uuid;

    #[test]
    fn record_to_bin_to_record() {
        let record = Record::new(Uuid::new_v4(), "type", "name", "data");
        let encoded = record.encode();
        let new_record = Record::decode(&encoded);
        assert!(record.entity_id == new_record.entity_id)
    }
}
