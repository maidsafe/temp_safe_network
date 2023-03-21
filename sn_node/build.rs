fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "rpc-service")]
    tonic_build::compile_protos("../sn_interface/proto/safenode/safenode.proto")?;

    Ok(())
}
