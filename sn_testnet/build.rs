fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "verify-nodes")]
    tonic_build::compile_protos("../sn_interface/proto/safenode/safenode.proto")?;

    Ok(())
}
