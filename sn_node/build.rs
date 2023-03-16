fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../sn_interface/proto/safenode/safenode.proto")?;
    Ok(())
}
