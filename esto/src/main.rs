//!
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use esto_core::{get_version, storage::Storage};
use std::path::PathBuf;

fn main() {
    println!("Esto, version: {}!", get_version());

    let _s = Storage::new(PathBuf::from("/tmp/test_i"), PathBuf::from("/tmp/test_d"));
}
