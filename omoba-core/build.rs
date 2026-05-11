fn main() {
    println!("cargo:rerun-if-changed=../proto/game.proto");

    #[cfg(feature = "grpc")]
    {
        tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(&["../proto/game.proto"], &["../proto"])
            .expect("Failed to compile proto files");
    }

    #[cfg(any(feature = "kcp", feature = "game-proto"))]
    {
        prost_build::compile_protos(&["../proto/game.proto"], &["../proto"])
            .expect("Failed to compile proto files");
    }
}
