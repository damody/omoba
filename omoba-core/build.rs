use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../proto/game.proto");
    println!("cargo:rerun-if-changed=src/generated/game.rs");
    println!("cargo:rerun-if-env-changed=OMOBA_UPDATE_PROTO_FALLBACK");

    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc unavailable");
    env::set_var("PROTOC", protoc);
    compile_with_protoc();

    if env::var_os("OMOBA_UPDATE_PROTO_FALLBACK").as_deref() == Some(std::ffi::OsStr::new("1")) {
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
        fs::copy(out_dir.join("game.rs"), "src/generated/game.rs")
            .expect("failed to update checked-in generated proto fallback");
    }
}

fn compile_with_protoc() {
    #[cfg(feature = "grpc")]
    {
        tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(&["../proto/game.proto"], &["../proto"])
            .expect("Failed to compile proto files");
    }

    #[cfg(not(feature = "grpc"))]
    {
        prost_build::compile_protos(&["../proto/game.proto"], &["../proto"])
            .expect("Failed to compile proto files");
    }
}
