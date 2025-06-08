use shared::crypto::{load_or_generate_keys, decrypt_layer};
use shared::packet::{HopData, ExitHop, GarlicPacket, Message};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncWriteExt, AsyncBufReadExt, BufReader}};
use base64::{engine::general_purpose, Engine as _};
use shared::crypto::{rsa_private_from_pem, rsa_public_to_pem, rsa_private_to_pem, rsa_public_from_pem, encrypt_layer, decrypt_layer as decrypt};
use std::path::Path;
use shared::config::load_node_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_node_config("exit_node/config.toml");
    let (priv_key, _pub_key) = load_or_generate_keys("exit_node/keys");
    let listener = TcpListener::bind(&config.listen).await?;
    println!("Exit node listening on {}", config.listen);
    loop {
        let (mut socket, _) = listener.accept().await?;
        let priv_key = priv_key.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(&priv_key, &mut socket).await {
                eprintln!("exit node error: {e}");
            }
        });
    }
}

fn load_service_key(domain: &str) -> rsa::RsaPrivateKey {
    let dir = Path::new("services").join(domain);
    std::fs::create_dir_all(&dir).ok();
    let priv_path = dir.join("private.pem");
    if priv_path.exists() {
        let pem = std::fs::read_to_string(priv_path).expect("read service key");
        rsa_private_from_pem(&pem)
    } else {
        let (priv_k, pub_k) = shared::crypto::generate_rsa_keys();
        std::fs::write(dir.join("private.pem"), rsa_private_to_pem(&priv_k)).unwrap();
        std::fs::write(dir.join("public.pem"), rsa_public_to_pem(&pub_k)).unwrap();
        priv_k
    }
}

async fn handle_client(priv_key: &rsa::RsaPrivateKey, socket: &mut TcpStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let layer: shared::crypto::OnionLayer = serde_json::from_str(&line.trim())?;
    let decrypted = decrypt_layer(layer, priv_key);
    let hop: ExitHop = serde_json::from_slice(&decrypted)?;
    println!("Exit -> delivering to {}", hop.domain);
    let service_priv = load_service_key(&hop.domain);
    let inner_layer: shared::crypto::OnionLayer = serde_json::from_slice(&general_purpose::STANDARD.decode(hop.inner)? )?;
    let garlic_bytes = decrypt(inner_layer, &service_priv);
    let packet: GarlicPacket = serde_json::from_slice(&garlic_bytes)?;
    for msg in packet.messages {
        println!("Service {} received: {}", msg.destination, msg.body);
    }
    Ok(())
}
