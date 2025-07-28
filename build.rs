fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/account/v1/account.proto")?;
    Ok(())
}
