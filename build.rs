fn main() {
    tonic_build::compile_protos("proto/datamap.proto")
        .unwrap_or_else(|e| panic!("Failed to compile proto {:?}", e));
}
