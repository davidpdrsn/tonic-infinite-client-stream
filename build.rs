fn main() {
    tonic_build::compile_protos("protocol/foo.proto").unwrap();
}
