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

use crate::record::Record;

use rocksdb::{ColumnFamilyDescriptor, MergeOperands, Options, DB};

#[derive(Debug)]
///
pub struct Storage {
    indexes: DB,
    data: DB,
}

fn idx_merger(k: &[u8], v: Option<&[u8]>, ops: &mut MergeOperands) -> Option<Vec<u8>> {
    None
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
    ///     PathBuf::from("/tmp/test_i"),
    ///     PathBuf::from("/tmp/test_d")
    /// );
    /// ```
    pub fn new(index_path: PathBuf, data_path: PathBuf) -> Self {
        let mut idx_cf_opts = Options::default();
        idx_cf_opts.set_merge_operator("add_record_index", idx_merger, None);

        let idx_cf = ColumnFamilyDescriptor::new("idx", idx_cf_opts);

        let data_cf_opts = Options::default();
        let data_cf = ColumnFamilyDescriptor::new("data", data_cf_opts);

        let mut db_options = Options::default();
        db_options.create_missing_column_families(true);
        db_options.create_if_missing(true);
        db_options.set_keep_log_file_num(10);

        Storage {
            indexes: DB::open_cf_descriptors(&db_options, &index_path, vec![idx_cf]).unwrap(),
            data: DB::open_cf_descriptors(&db_options, &data_path, vec![data_cf]).unwrap(),
        }
    }

    ///
    pub fn write(&mut self, record: &Record) -> Result<(), &'static str> {
        // Write the data record, and get the ID

        // Merge the ID into the Index at record.entity_id
        let cf = self.indexes.cf_handle("idx").unwrap();
        self.indexes
            .merge_cf(
                cf,
                record.entity_id.to_hyphenated().to_string(),
                record.encode(),
            )
            .unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Storage;

    use test_dir::{DirBuilder, FileType, TestDir};

    #[test]
    fn can_create_storage() {
        let tmp = TestDir::temp()
            .create("idx", FileType::Dir)
            .create("dta", FileType::Dir);

        let _ = Storage::new(tmp.path("idx"), tmp.path("dta"));
    }
}
