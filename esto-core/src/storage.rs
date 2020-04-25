use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use std::path::PathBuf;

pub struct Storage {
    indexes: DB,
    data: DB,
}

impl Storage {
    fn new(index_path: PathBuf, data_path: PathBuf) -> Self {
        let idx_cf_opts = Options::default();
        let idx_cf = ColumnFamilyDescriptor::new("idx", idx_cf_opts);

        let data_cf_opts = Options::default();
        let data_cf = ColumnFamilyDescriptor::new("data", data_cf_opts);

        let mut db_options = Options::default();
        db_options.create_missing_column_families(true);
        db_options.create_if_missing(true);

        Storage {
            indexes: DB::open_cf_descriptors(&db_options, &index_path, vec![idx_cf]).unwrap(),
            data: DB::open_cf_descriptors(&db_options, &data_path, vec![data_cf]).unwrap(),
        }
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

        let s = Storage::new(tmp.path("idx"), tmp.path("dta"));
    }
}
