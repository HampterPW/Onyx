use serde::{Serialize, Deserialize};
use crate::crypto::OnionLayer;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub destination: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GarlicPacket {
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnionPacket {
    pub layers: Vec<OnionLayer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HopData {
    pub next: Option<String>,
    pub inner: String, // base64 of next layer or garlic packet
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExitHop {
    pub domain: String,
    pub inner: String, // base64 of garlic packet encrypted with service key
}
