use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a new Ed25519 keypair for package signing
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut secret = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

/// Sign data with Ed25519 private key
pub fn sign(private_key_bytes: &[u8], data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let key_bytes: [u8; 32] = private_key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid private key length"))?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Verify Ed25519 signature
pub fn verify(public_key_bytes: &[u8], data: &[u8], signature_bytes: &[u8]) -> anyhow::Result<bool> {
    let key_bytes: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
    let sig_bytes: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;

    let verifying_key = VerifyingKey::from_bytes(&key_bytes)?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    Ok(verifying_key.verify(data, &signature).is_ok())
}

/// Compute SHA-256 checksum
pub fn sha256_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key) = generate_keypair();
        let data = b"test data to sign";
        let signature = sign(&private_key, data).unwrap();
        assert!(verify(&public_key, data, &signature).unwrap());
    }

    #[test]
    fn test_sha256_checksum() {
        let hash = sha256_checksum(b"hello");
        assert_eq!(hash.len(), 64); // hex-encoded SHA-256 is 64 chars
    }
}
