fn main() {
    prost_build::compile_protos(&["proto/fixture.proto"], &["proto"]).unwrap();
    println!("cargo:rerun-if-changed=proto");
}
