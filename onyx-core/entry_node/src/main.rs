use shared::crypto::{load_or_generate_keys, decrypt_layer};
use shared::packet::HopData;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncWriteExt, AsyncBufReadExt, BufReader}};
use base64::{engine::general_purpose, Engine as _};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (priv_key, _pub_key) = load_or_generate_keys("entry_node/keys");
    let listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("Entry node listening on 7000");
    loop {
        let (mut socket, _) = listener.accept().await?;
        let priv_key = priv_key.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(&priv_key, &mut socket).await {
                eprintln!("entry node error: {e}");
            }
        });
    }
}

async fn handle_client(priv_key: &rsa::RsaPrivateKey, socket: &mut TcpStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let layer: shared::crypto::OnionLayer = serde_json::from_str(&line.trim())?;
    let decrypted = decrypt_layer(layer, priv_key);
    let hop: HopData = serde_json::from_slice(&decrypted)?;
    if let Some(next) = hop.next {
        println!("Entry -> forwarding to {next}");
        let inner = general_purpose::STANDARD.decode(hop.inner)?;
        let mut stream = TcpStream::connect(next).await?;
        stream.write_all(&inner).await?;
        stream.write_all(b"\n").await?;
    }
    Ok(())
}
