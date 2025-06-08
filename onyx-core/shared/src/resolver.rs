use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use rsa::RsaPublicKey;
use crate::crypto::rsa_public_from_pem;

static RESOLVER: Lazy<Mutex<HashMap<String, RsaPublicKey>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn load_resolver(path: &str) {
    let data = fs::read_to_string(path).expect("resolver file");
    let mut map = RESOLVER.lock().unwrap();
    for line in data.lines() {
        if let Some((domain, key_pem)) = line.split_once(' ') {
            map.insert(domain.to_string(), rsa_public_from_pem(key_pem));
        }
    }
}

pub fn resolve(domain: &str) -> Option<RsaPublicKey> {
    let map = RESOLVER.lock().unwrap();
    map.get(domain).cloned()
}
