fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "node-ctl")]
    // Note this requires the `protoc` compiler to be installed on the host system,
    // refer to https://grpc.io/docs/protoc-installation for installation guidance.
    tonic_build::compile_protos("../sn_interface/proto/safenode/safenode.proto")?;

    Ok(())
}
