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
    // unused_qualifications - grpc generated code triggers this
)]

use esto_core::{get_version, storage::Storage};
use std::path::PathBuf;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Esto, version: {}!", get_version());
    let _s = Storage::new(PathBuf::from("/tmp/test_i"), PathBuf::from("/tmp/test_d"));

    server::run().await.unwrap();

    Ok(())
}
