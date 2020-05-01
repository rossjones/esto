//!
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize, Serialize)]
///A single event record
pub struct Record<'a> {
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
    fn new(
        entity_id: Uuid,
        entity_type: &'a str,
        event_name: &'a str,
        event_data: &'a str,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        Record {
            timestamp: ts,
            entity_id,
            entity_type,
            event_name,
            event_data,
        }
    }

    /// Converts bytes (from storage) into a record
    pub fn decode(val: &'a [u8]) -> Self {
        bincode::deserialize(&val[..]).unwrap()
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
        let mut record = Record::new(Uuid::new_v4(), "type", "name", "data");
        let encoded: _ = record.encode();
        let new_record = Record::decode(&encoded);
        assert!(record.entity_id == new_record.entity_id)
    }
}
