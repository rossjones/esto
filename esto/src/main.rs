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
use std::cell::Cell;
use std::path::PathBuf;
use tokio::sync::oneshot;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Esto, version: {}!", get_version());

    let (tx, rx) = oneshot::channel::<()>();
    let txr = Cell::new(Some(tx));

    ctrlc::set_handler(move || {
        println!("Exiting...");
        txr.take().unwrap().send(()).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    let s = Storage::new(PathBuf::from("/tmp/testing")).unwrap();
    server::run(s, rx).await.unwrap();

    Ok(())
}
