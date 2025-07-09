fn main() {
    tonic_build::compile_protos("proto/rawst.proto").unwrap();
}