use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit, OsRng, generic_array::GenericArray}};
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey}};
use rand::rngs::OsRng as RandOsRng;
use rsa::Oaep;
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};

pub fn generate_rsa_keys() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = RandOsRng;
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate key");
    let pub_key = RsaPublicKey::from(&priv_key);
    (priv_key, pub_key)
}

pub fn rsa_public_to_pem(pub_key: &RsaPublicKey) -> String {
    pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).expect("pem")
}

pub fn rsa_private_to_pem(priv_key: &RsaPrivateKey) -> String {
    priv_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).expect("pem").to_string()
}

pub fn rsa_private_from_pem(pem: &str) -> RsaPrivateKey {
    RsaPrivateKey::from_pkcs8_pem(pem).expect("invalid pem")
}

pub fn rsa_public_from_pem(pem: &str) -> RsaPublicKey {
    RsaPublicKey::from_public_key_pem(pem).expect("invalid pem")
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OnionLayer {
    pub encrypted_key: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub fn encrypt_layer(data: &[u8], pub_key: &RsaPublicKey) -> OnionLayer {
    let aes_key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&aes_key);
    let nonce = aes_gcm::Nonce::from_slice(&rand::random::<[u8; 12]>());
    let ciphertext = cipher.encrypt(nonce, data).expect("encrypt");
    let mut rng = RandOsRng;
    let enc_key = pub_key.encrypt(&mut rng, Oaep::new::<Sha256>(), &aes_key).expect("rsa enc");
    OnionLayer {
        encrypted_key: general_purpose::STANDARD.encode(enc_key),
        nonce: general_purpose::STANDARD.encode(nonce),
        ciphertext: general_purpose::STANDARD.encode(ciphertext),
    }
}

pub fn decrypt_layer(layer: OnionLayer, priv_key: &RsaPrivateKey) -> Vec<u8> {
    let enc_key = general_purpose::STANDARD.decode(layer.encrypted_key).expect("b64");
    let aes_key = priv_key.decrypt(Oaep::new::<Sha256>(), &enc_key).expect("dec key");
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&aes_key));
    let nonce_bytes = general_purpose::STANDARD.decode(layer.nonce).expect("b64");
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let ciphertext = general_purpose::STANDARD.decode(layer.ciphertext).expect("b64");
    cipher.decrypt(nonce, ciphertext.as_ref()).expect("decrypt")
}
use std::fs;
use std::path::Path;

pub fn load_or_generate_keys(dir: &str) -> (RsaPrivateKey, RsaPublicKey) {
    let priv_path = Path::new(dir).join("private.pem");
    let pub_path = Path::new(dir).join("public.pem");
    if priv_path.exists() && pub_path.exists() {
        let priv_pem = fs::read_to_string(&priv_path).expect("read priv");
        let pub_pem = fs::read_to_string(&pub_path).expect("read pub");
        let priv_key = rsa_private_from_pem(&priv_pem);
        let pub_key = rsa_public_from_pem(&pub_pem);
        (priv_key, pub_key)
    } else {
        let (priv_key, pub_key) = generate_rsa_keys();
        fs::create_dir_all(dir).ok();
        fs::write(&priv_path, rsa_private_to_pem(&priv_key)).expect("write");
        fs::write(&pub_path, rsa_public_to_pem(&pub_key)).expect("write");
        (priv_key, pub_key)
    }
}
