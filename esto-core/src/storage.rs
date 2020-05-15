//! Storage Engine for Esto
//!
//! The storage engine is responsible for persisting and reading data
//! that it is given.  The underlying storage is a KV store, and because
//! we want to record lots of records with the same ID (e.g we are
//! storing small updates to a specific record/thing) that we need to
//! extract later, we are using the following pattern.
//!
//! ## The indexes column family
//!
//! The indexes column family store the indexes into the data column
//! family for a specific record.  For each record we will use its ID
//! as the key into the indexes, where the value will be a list of
//! {data_id, timestamp}. Each data_id will be a record that is stored
//! in the data column family.
//!
//! ## The data column family
//!
//! This column family stores records against a UUID identifier. Some
//! small amount of the data is 'book-keeping' with the rest being the
//! JSON data provided by the client.
//!
//! ## Internal structures
//!
//! TODO: Define the book-keeping structures
//!
//! ## Patterns
//!
//! When writing a record, for example:
//!
//! ```json
//! {
//!   'event': 'OrderCancelled',
//!   'object_id': '1111-1111-1111-1111',
//!   'data':  { ... }
//! }
//! ```
//!
//! the storage engine will have to execute the following steps inside
//! a transaction:
//!
//! * Write record to data cf with a new uuid key
//! * Merge an index records into the indexes cf for key=record.object_id
//!   containing the current time and the new uuid key from the previous step.
//!
//!
//!
//! TODO: Do we want to partition the data more finely in CFs? Should we
//! shard the IDs across multiple CFs?
use std::path::PathBuf;

use crate::errors::{StorageError, StorageResult};
use crate::{index::Index, record::Record};

use rocksdb::{ColumnFamilyDescriptor, MergeOperands, Options, DB};
use uuid::Uuid;

/// The Storage struct represents a single store and attempts to
/// abstract away the details of storage as much as possible.
pub struct Storage {
    data: DB,
}

fn idx_merger(
    key: &[u8],
    existing: Option<&[u8]>,
    operands: &mut MergeOperands,
) -> Option<Vec<u8>> {
    let entity_id = Uuid::from_slice(key).unwrap();

    //  panicked at 'called `Result::unwrap()` on an `Err` value: Error(Build(Error { expected: 16, found: 116 }))'
    let new_record_ids: Vec<Uuid> = operands
        .map(|op| {
            // op MUST be 16 bytes
            if op.len() != 16 {
                let r = Record::decode(&op);
                println!("{:?}", r);
            }
            Uuid::from_slice(&op).unwrap()
        })
        .collect();

    let mut index = match existing {
        // If there is no existing data, then create a new Index
        None => Index::new(),
        // Otherwise decode the index from the existing data
        Some(val) => Index::decode(entity_id, val),
    };

    // Append each new record ID into the index
    new_record_ids
        .into_iter()
        .for_each(|u| index.append_record(u));
    Some(index.encode())
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("indexes", &self.data)
            .finish()
    }
}

impl Storage {
    /// Creates a new storage engine
    ///
    /// The creation of the storage engine is very trusting, and
    /// currently no attempt is made to ensure the existence of the
    /// paths provided.
    ///
    /// # Arguments
    ///
    ///  * `index_path` - The folder where the index data is stored
    /// * `data_path` - The folder where the data itself is held
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use esto_core::storage::Storage;
    ///
    /// let s = Storage::new(
    ///     PathBuf::from("/tmp/test_d")
    /// ).unwrap();
    /// ```
    pub fn new(data_path: PathBuf) -> StorageResult<Self> {
        let mut idx_cf_opts = Options::default();
        idx_cf_opts.set_merge_operator("add_record_index", idx_merger, None);

        let idx_cf = ColumnFamilyDescriptor::new("idx", idx_cf_opts);

        let data_cf_opts = Options::default();
        let data_cf = ColumnFamilyDescriptor::new("data", data_cf_opts);

        let mut db_options = Options::default();
        db_options.create_missing_column_families(true);
        db_options.create_if_missing(true);
        db_options.set_keep_log_file_num(10);

        Ok(Storage {
            data: DB::open_cf_descriptors(&db_options, &data_path, vec![idx_cf, data_cf]).unwrap(),
        })
    }

    /// Writes a new record to the data column-family, and then updates the
    /// appopriate index by appending the new record's ID.
    pub fn write(&self, record: &Record<'_>) -> StorageResult<()> {
        // Write the data record ...
        let cf_data = self.data.cf_handle("data").unwrap();
        self.data
            .put_cf(cf_data, record.id.as_bytes(), record.encode())
            .unwrap();

        // Merge the ID of the record onto the tail of the index.
        let cf_idx = self.data.cf_handle("idx").unwrap();

        self.data
            .merge_cf(cf_idx, record.entity_id.as_bytes(), record.id.as_bytes())
            .unwrap();

        self.data.flush_cf(&cf_data).unwrap();
        self.data.flush_cf(&cf_idx).unwrap();

        Ok(())
    }

    /// Attempts to read all of the records for a specific entity,
    /// returning the list of events as a vec of vec[u8]. It is
    /// the responsibility of the caller to convert this to a
    /// Record struct
    pub fn read(&self, entity_id: Uuid) -> StorageResult<Vec<Vec<u8>>> {
        let idx = self.get_index(entity_id);

        let record_ids = match idx {
            Ok(i) => i.records,
            Err(err) => return Err(err),
        };

        // Erm, what no multiget?
        let cf_data = self.data.cf_handle("data").unwrap();

        let records = record_ids
            .into_iter()
            .map(|r| {
                // If this record isn't here, it means we failed to write it but
                // did manage to update the index...
                self.data.get_cf(cf_data, r.0.as_bytes()).unwrap().unwrap()
            })
            .collect();
        Ok(records)
    }

    /// Retrieves an Index given the id of an entity
    pub fn get_index(&self, entity_id: Uuid) -> StorageResult<Index> {
        let cf_idx = self.data.cf_handle("idx").unwrap();

        let res = self.data.get_cf(cf_idx, entity_id.as_bytes()).unwrap();

        let v_data = match res {
            Some(data) => data,
            None => {
                return Err(StorageError {
                    message: "Failed to find index data",
                })
            }
        };

        Ok(Index::decode(entity_id, &v_data))
    }
}

#[cfg(test)]
mod tests {
    use super::Storage;

    use crate::record::Record;

    use test_dir::{DirBuilder, FileType, TestDir};
    use uuid::Uuid;

    #[test]
    fn can_write_a_record() {
        let tmp = TestDir::temp().create("dta", FileType::Dir);

        let storage = Storage::new(tmp.path("dta")).unwrap();
        let record = Record::new(Uuid::new_v4(), "type", "name", "data");

        storage.write(&record).unwrap();

        let idx = storage.get_index(record.entity_id).unwrap();
        assert_eq!(idx.records.len(), 1);

        let recs = storage.read(idx.id).unwrap();
        assert_eq!(recs.len(), 1);

        let record = Record::decode(&recs[0]);
        assert_eq!(record.entity_id, idx.id);
        assert_eq!(record.entity_type, "type");
        assert_eq!(record.event_name, "name");
        assert_eq!(record.event_data, "data");
    }

    #[test]
    fn can_write_two_records() {
        let tmp = TestDir::temp().create("dta", FileType::Dir);

        let storage = Storage::new(tmp.path("dta")).unwrap();

        let record1 = Record::new(Uuid::new_v4(), "type1", "name1", "data1");
        let record2 = Record::new(record1.entity_id, "type2", "name2", "data2");

        storage.write(&record1).unwrap();
        storage.write(&record2).unwrap();

        assert_eq!(record1.entity_id, record2.entity_id);

        let idx = storage.get_index(record1.entity_id).unwrap();
        assert_eq!(idx.records.len(), 2);
        assert_eq!(idx.records[0].0, record1.id);
        assert_eq!(idx.records[1].0, record2.id);
        // Check timestamps
        assert!(idx.records[0].1 < idx.records[1].1);

        let recs = storage.read(idx.id).unwrap();
        assert_eq!(recs.len(), 2);

        let mut record = Record::decode(&recs[0]);
        assert_eq!(record.entity_id, idx.id);
        assert_eq!(record.entity_type, "type1");
        assert_eq!(record.event_name, "name1");
        assert_eq!(record.event_data, "data1");

        record = Record::decode(&recs[1]);
        assert_eq!(record.entity_id, idx.id);
        assert_eq!(record.entity_type, "type2");
        assert_eq!(record.event_name, "name2");
        assert_eq!(record.event_data, "data2");
    }
}
