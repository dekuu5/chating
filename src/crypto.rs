use std::path::Path;

use openssl::rand::rand_bytes;
use openssl::rsa::{Padding, Rsa};
use openssl::symm::{Cipher, decrypt, encrypt};


#[derive(Debug)]
pub struct RSAKeys {
    pub_key: Vec<u8>,
    pri_key: Vec<u8>
}

#[derive(Debug, PartialEq, Eq)]
pub struct AESKeys {
    key: Vec<u8>,
    iv: Vec<u8>
}



impl RSAKeys {
    pub fn new() -> Self {
        let rsa = Rsa::generate(2048).unwrap();
        let pri_key = rsa.private_key_to_pem().unwrap();
        let pub_key = rsa.public_key_to_pem().unwrap();
        Self { pub_key, pri_key }
    }
    // TODO: add a way to load keys from desk 
    pub fn from_desk(path: &Path) -> Self{
        todo!()
    }

    pub fn encrypt_public(&self, plain:& Vec<u8>) -> Vec<u8> {
        let rsa = Rsa::public_key_from_pem(&self.pub_key).unwrap();
        let mut cipher = vec![0u8; rsa.size() as usize];
        let len = rsa.public_encrypt(plain, &mut cipher, Padding::PKCS1_OAEP).unwrap();
        cipher.truncate(len);
        cipher
    }
    pub fn decrypt_public(&self, cipher:& Vec<u8>) -> Vec<u8> {
        let rsa = Rsa::public_key_from_pem(&self.pub_key).unwrap();
        let mut plain = vec![0u8; rsa.size() as usize];
        let len = rsa.public_decrypt(cipher, &mut plain, Padding::PKCS1).unwrap();
        plain.truncate(len);
        plain
    }
    pub fn encrypt_private(&self, plain:& Vec<u8>) -> Vec<u8> {
        let rsa = Rsa::private_key_from_pem(&self.pri_key).unwrap();
        let mut cipher = vec![0u8; rsa.size() as usize];
        let len = rsa.private_encrypt(plain, &mut cipher, Padding::PKCS1).unwrap();
        cipher.truncate(len);
        cipher
    }
    pub fn decrypt_private(&self, cipher:& Vec<u8>) -> Vec<u8> {
        let rsa = Rsa::private_key_from_pem(&self.pri_key).unwrap();
        let mut plain = vec![0u8; rsa.size() as usize];
        let len = rsa.private_decrypt(cipher, &mut plain, Padding::PKCS1_OAEP).unwrap();
        plain.truncate(len);
        plain
    }
}

impl AESKeys {
    pub fn new() -> Self {
        let mut key = vec![0u8;32];
        let mut iv = vec![0u8;16];
        rand_bytes(&mut key).unwrap();
        rand_bytes(&mut iv).unwrap();
        Self { key, iv }
    }

    pub fn from_bytes(bytes: & Vec<u8>) -> Result<Self,String> {
        if bytes.len() != (32 + 16) {
            return Err("AES key and IV should equals a 48 bytes long".to_owned());
        }
        let mut key = vec![0u8;32];
        let mut iv = vec![0u8;16];
        bytes[..32].clone_into(&mut key);
        bytes[32..].clone_into(&mut iv);
        Ok(
            Self { key, iv }
        )
    }

    pub fn encrypt(&self, plain:& Vec<u8>) -> Vec<u8> {
        encrypt(Cipher::aes_256_cbc(), &self.key, Some(&self.iv), plain).unwrap()
    }
    pub fn decrypt(&self, cipher:& Vec<u8>) -> Vec<u8> {
        decrypt(Cipher::aes_256_cbc(), &self.key, Some(&self.iv), cipher).unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.key);
        bytes.extend_from_slice(&self.iv);
        bytes
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pub_encrypt() {
        let rsa_keys = RSAKeys::new();
        let plain = "nice_test".as_bytes();
        let cipher = rsa_keys.encrypt_public(&plain.to_vec());
        let new_plain = rsa_keys.decrypt_private(&cipher);
        assert_eq!(new_plain, plain);
    }

    #[test]
    fn test_pri_encrypt() {
        let rsa_keys = RSAKeys::new();
        let plain = "nice_test".as_bytes();
        println!("{:?}",plain);
        let cipher = rsa_keys.encrypt_private(&plain.to_vec());
        let new_plain = rsa_keys.decrypt_public(&cipher);
        println!("{:?}",new_plain);
        assert_eq!(new_plain, plain);
    }


    #[test]
    fn test_from_to_bytes_flow() {
        let aes_keys = AESKeys::new();

        let bytes = aes_keys.to_bytes();
        let new_keys = AESKeys::from_bytes(&bytes).unwrap();

        assert_eq!(new_keys,aes_keys);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let aes_keys = AESKeys::new();

        let plain = "nice_secret_massage".as_bytes();

        let cipher = aes_keys.encrypt(&plain.to_vec());
        let new_plain = aes_keys.decrypt(&cipher);
        assert_eq!(plain, new_plain);
        
    }
}