//! An index containing an identifier and a vector of record ids.
//!
//! An index should have the id of the 'thing' that it is indexing
//! and then a vector of records ids.  Each of these ids will point
//! to a record in the data column family containing the data for
//! that event.
//!
//! The ordering of the records in this struct is the ordering that
//! the events were received in.
//!
//! ## Example
//!
//! Create a new index, convert it to binary,
//! convert it back to an index.
//!
//! ```
//! use esto_core::index::Index;
//!
//! let mut idx = Index::new();
//! let encoded: _ = idx.encode();
//! let new_idx = Index::decode(idx.id, &encoded);
//! assert!(idx.records == new_idx.records)
//! ```
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::time::{Duration, SystemTime};

// A RecordLink is a unique id and a timestamp.  Currently the
// timestamp is from a physical clock. Not even monotonic :(
// TODO: Switch to Logical clocks ftw
type RecordLink = (Uuid, Duration);

#[derive(Debug, Serialize, Deserialize)]
/// An index for an entity
pub struct Index {
    /// The identifier for the entity being found
    pub id: Uuid,

    /// An ordered list of records ids with timestamps.  The timestamps
    /// in these records must be in order.
    pub records: Vec<RecordLink>,
}

impl Index {
    /// Creates a new index with a new identifier and no record links.
    pub fn new() -> Self {
        Index {
            id: Uuid::new_v4(),
            records: vec![],
        }
    }

    /// Appends the ID of a record and sets the timstamp to number of seconds
    /// since the epoch
    pub fn append_record(&mut self, uuid: Uuid) {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        self.records.push((uuid, ts));
    }

    /// Creates a new Index by being provided an ID and deserializing the
    /// provided binary into a Vector of RecordLinks which are stored in
    /// the new Index.
    pub fn decode(id: Uuid, val: &[u8]) -> Self {
        Index {
            id,
            records: bincode::deserialize(&val[..]).unwrap(),
        }
    }

    /// Encode the records in this Index into Vector of bytes
    pub fn encode(&mut self) -> Vec<u8> {
        bincode::serialize(&self.records).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Index;

    use uuid::Uuid;

    #[test]
    fn index_to_bin_to_index() {
        let mut i = Index::new();
        let v: _ = i.encode();
        let new_i = Index::decode(i.id, &v);
        assert!(i.records == new_i.records)
    }

    #[test]
    fn index_to_bin_to_index_with_record() {
        let u = Uuid::new_v4();

        let mut i = Index::new();
        i.append_record(u);

        let v: _ = i.encode();
        let new_i = Index::decode(i.id, &v);

        assert!(new_i.records.len() == 1);
        assert!(i.records.len() == new_i.records.len());
    }
}
