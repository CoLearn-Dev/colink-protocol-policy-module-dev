fn main() {
    prost_build::compile_protos(&["proto/colink_policy_module.proto"], &["proto/"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {e:?}"));
}
