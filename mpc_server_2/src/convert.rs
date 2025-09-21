use ed25519_dalek::{SigningKey, VerifyingKey};

pub fn keypair_from_base64_strings(private_key_b64: &str, public_key_b64: &str) -> Result<(SigningKey, VerifyingKey), Box<dyn std::error::Error>> {
    use base64::{Engine as _, engine::general_purpose};
    
    let private_key_bytes = general_purpose::STANDARD.decode(private_key_b64)?;
    let signing_key = SigningKey::from_bytes(&private_key_bytes.try_into().map_err(|_| "Invalid private key length")?);
    
    let public_key_bytes = general_purpose::STANDARD.decode(public_key_b64)?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().map_err(|_| "Invalid public key length")?)?;
    
    Ok((signing_key, verifying_key))
}