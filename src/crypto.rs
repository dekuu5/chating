use std::fs;
use std::path::{Path, PathBuf};

use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rand::rand_bytes;
use openssl::rsa::{Padding, Rsa};
use openssl::sign::{Signer, Verifier};
use openssl::symm::{Cipher, decrypt, encrypt};


pub struct PrivKey {
    rsa: Rsa<Private>,
}

pub struct PubKey {
    rsa: Rsa<Public>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AESKeys {
    key: Vec<u8>,
    iv: Vec<u8>,
}


pub fn generate_keypair(out_dir: &Path, name: &str) -> Result<(PathBuf, PathBuf), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("failed to create out dir: {e}"))?;

    let rsa = Rsa::generate(2048)
        .map_err(|e| format!("failed to generate rsa: {e}"))?;

    let priv_pem = rsa.private_key_to_pem()
        .map_err(|e| format!("failed to serialize private key: {e}"))?;
    let pub_pem = rsa.public_key_to_pem()
        .map_err(|e| format!("failed to serialize public key: {e}"))?;

    let priv_path = out_dir.join(format!("{name}_priv.pem"));
    let pub_path = out_dir.join(format!("{name}_pub.pem"));

    fs::write(&priv_path, &priv_pem)
        .map_err(|e| format!("failed to write private key: {e}"))?;
    fs::write(&pub_path, &pub_pem)
        .map_err(|e| format!("failed to write public key: {e}"))?;

    Ok((priv_path, pub_path))
}


impl PrivKey {
    pub fn load(path: &Path) -> Result<Self, String> {
        let pem = fs::read(path)
            .map_err(|e| format!("failed to read private key {}: {e}", path.display()))?;
        let rsa = Rsa::private_key_from_pem(&pem)
            .map_err(|e| format!("failed to parse private key: {e}"))?;
        Ok(Self { rsa })
    }

    pub fn decrypt(&self, cipher: &[u8]) -> Result<Vec<u8>, String> {
        let mut plain = vec![0u8; self.rsa.size() as usize];
        let len = self.rsa.private_decrypt(cipher, &mut plain, Padding::PKCS1_OAEP)
            .map_err(|e| format!("rsa decrypt failed: {e}"))?;
        plain.truncate(len);
        Ok(plain)
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let pkey = PKey::from_rsa(self.rsa.clone())
            .map_err(|e| format!("pkey wrap failed: {e}"))?;
        let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
            .map_err(|e| format!("signer init failed: {e}"))?;
        signer.update(data)
            .map_err(|e| format!("signer update failed: {e}"))?;
        signer.sign_to_vec()
            .map_err(|e| format!("signing failed: {e}"))
    }
}

impl PubKey {
    pub fn load(path: &Path) -> Result<Self, String> {
        let pem = fs::read(path)
            .map_err(|e| format!("failed to read public key {}: {e}", path.display()))?;
        let rsa = Rsa::public_key_from_pem(&pem)
            .map_err(|e| format!("failed to parse public key: {e}"))?;
        Ok(Self { rsa })
    }

    pub fn encrypt(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
        let mut cipher = vec![0u8; self.rsa.size() as usize];
        let len = self.rsa.public_encrypt(plain, &mut cipher, Padding::PKCS1_OAEP)
            .map_err(|e| format!("rsa encrypt failed: {e}"))?;
        cipher.truncate(len);
        Ok(cipher)
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> Result<bool, String> {
        let pkey = PKey::from_rsa(
            Rsa::public_key_from_pem(
                &self.rsa.public_key_to_pem()
                    .map_err(|e| format!("pubkey serialize failed: {e}"))?
            ).map_err(|e| format!("pubkey reparse failed: {e}"))?
        ).map_err(|e| format!("pkey wrap failed: {e}"))?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey)
            .map_err(|e| format!("verifier init failed: {e}"))?;
        verifier.update(data)
            .map_err(|e| format!("verifier update failed: {e}"))?;
        verifier.verify(sig)
            .map_err(|e| format!("verify failed: {e}"))
    }
}

impl AESKeys {
    pub fn new() -> Self {
        let mut key = vec![0u8; 32];
        let mut iv = vec![0u8; 16];
        rand_bytes(&mut key).unwrap();
        rand_bytes(&mut iv).unwrap();
        Self { key, iv }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != (32 + 16) {
            return Err("AES key and IV must be 48 bytes total".to_owned());
        }
        Ok(Self {
            key: bytes[..32].to_vec(),
            iv: bytes[32..].to_vec(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(&self.key);
        bytes.extend_from_slice(&self.iv);
        bytes
    }

    pub fn encrypt(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
        encrypt(Cipher::aes_256_cbc(), &self.key, Some(&self.iv), plain)
            .map_err(|e| format!("aes encrypt failed: {e}"))
    }

    pub fn decrypt(&self, cipher: &[u8]) -> Result<Vec<u8>, String> {
        decrypt(Cipher::aes_256_cbc(), &self.key, Some(&self.iv), cipher)
            .map_err(|e| format!("aes decrypt failed: {e}"))
    }
}


pub fn random_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; 32];
    rand_bytes(&mut nonce).unwrap();
    nonce
}

pub fn hash_sha256(data: &[u8]) -> Result<Vec<u8>, String> {
    openssl::hash::hash(MessageDigest::sha256(), data)
        .map(|h| h.to_vec())
        .map_err(|e| format!("sha256 failed: {e}"))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_roundtrip() {
        let tmp = std::env::temp_dir().join("chating_test_keys");
        let _ = fs::remove_dir_all(&tmp);
        let (priv_path, pub_path) = generate_keypair(&tmp, "A").unwrap();

        let priv_key = PrivKey::load(&priv_path).unwrap();
        let pub_key = PubKey::load(&pub_path).unwrap();

        let plain = b"hello world";
        let cipher = pub_key.encrypt(plain).unwrap();
        let recovered = priv_key.decrypt(&cipher).unwrap();
        assert_eq!(recovered, plain);

        let sig = priv_key.sign(plain).unwrap();
        assert!(pub_key.verify(plain, &sig).unwrap());
        assert!(!pub_key.verify(b"tampered", &sig).unwrap());
    }

    #[test]
    fn test_aes_roundtrip() {
        let keys = AESKeys::new();
        let plain = b"nice_secret_message";
        let cipher = keys.encrypt(plain).unwrap();
        let recovered = keys.decrypt(&cipher).unwrap();
        assert_eq!(recovered, plain);
    }

    #[test]
    fn test_aes_to_from_bytes() {
        let keys = AESKeys::new();
        let bytes = keys.to_bytes();
        let recovered = AESKeys::from_bytes(&bytes).unwrap();
        assert_eq!(recovered, keys);
    }
}
