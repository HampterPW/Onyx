use shared::crypto::{load_or_generate_keys, encrypt_layer, rsa_public_from_pem};
use shared::packet::{GarlicPacket, Message, HopData, ExitHop};
use tokio::{net::TcpStream, io::AsyncWriteExt};
use base64::{engine::general_purpose, Engine as _};
use shared::resolver::{load_resolver, resolve};
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load service resolution file
    load_resolver("services.txt");

    // load node public keys
    let entry_pub = rsa_public_from_pem(&fs::read_to_string("entry_node/keys/public.pem")?);
    let middle_pub = rsa_public_from_pem(&fs::read_to_string("middle_node/keys/public.pem")?);
    let exit_pub = rsa_public_from_pem(&fs::read_to_string("exit_node/keys/public.pem")?);

    // messages to send
    let msgs = vec![
        Message { destination: "user1".into(), body: "Hello 1".into() },
        Message { destination: "user2".into(), body: "Hello 2".into() },
    ];

    let service_domain = "echo.ony";
    let service_pub = resolve(service_domain).expect("service not found");

    let garlic = GarlicPacket { messages: msgs };
    let garlic_json = serde_json::to_vec(&garlic)?;
    let service_layer = encrypt_layer(&garlic_json, &service_pub);
    let service_layer_json = serde_json::to_vec(&service_layer)?;

    let exit_hop = ExitHop { domain: service_domain.to_string(), inner: general_purpose::STANDARD.encode(service_layer_json) };
    let exit_hop_json = serde_json::to_vec(&exit_hop)?;
    let exit_layer = encrypt_layer(&exit_hop_json, &exit_pub);
    let exit_layer_json = serde_json::to_vec(&exit_layer)?;

    let middle_hop = HopData { next: Some("127.0.0.1:7002".into()), inner: general_purpose::STANDARD.encode(exit_layer_json) };
    let middle_hop_json = serde_json::to_vec(&middle_hop)?;
    let middle_layer = encrypt_layer(&middle_hop_json, &middle_pub);
    let middle_layer_json = serde_json::to_vec(&middle_layer)?;

    let entry_hop = HopData { next: Some("127.0.0.1:7001".into()), inner: general_purpose::STANDARD.encode(middle_layer_json) };
    let entry_layer = encrypt_layer(&serde_json::to_vec(&entry_hop)?, &entry_pub);

    let packet = serde_json::to_string(&entry_layer)?;
    let mut stream = TcpStream::connect("127.0.0.1:7000").await?;
    stream.write_all(packet.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    println!("Client sent packet");
    Ok(())
}
