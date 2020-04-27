use std::time::Duration;
use uuid::Uuid;

pub struct Record {
    // Number of sends since the epoch
    pub timestamp: Duration,

    // The ID of the entity, which is essentially a link back to the
    // Index that points to this record.
    pub entity_id: Uuid,

    // The string representation of the entity's type
    pub entity_type: &str,

    // The name of the event
    pub event_name: &str,

    // The event data itself, probably in JSON.
    pub event_data: &str,
}

impl Record {
    fn new(entity_id: Uuid, entity_type: &str, event_name: &str, event_data: &str) -> Self {
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
}
