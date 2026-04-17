fn main() {
    prost_build::compile_protos(&["../proto/game.proto"], &["../proto"])
        .expect("Failed to compile proto files");
}
