fn main() {
    println!("cargo:rerun-if-changed=proto/*.proto");

    tonic_build::configure()
        .build_server(true)
        .compile(&["proto/store.proto"], &["proto"])
        .unwrap();
}
