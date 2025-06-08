# Onyx Routing System

This is a minimal demonstration of an anonymizing router combining onion and garlic routing principles using Rust.

## Structure
- `shared` – common cryptography utilities, packet types and service resolver.
- `entry_node`, `middle_node`, `exit_node` – three hop nodes that decrypt a layer and forward the packet.
- `client` – builds an onion/garlic packet and sends it through the network.

## Running
1. Ensure you have Rust installed.
2. Build the workspace:
   ```bash
   cargo build --workspace
   ```
3. In separate terminals, run each node:
   ```bash
   cargo run -p entry_node
   cargo run -p middle_node
   cargo run -p exit_node
   ```
4. After starting the exit node for the first time, copy its generated public key into a `services.txt` file:
   ```
   echo.ony <contents of exit_node/services/echo.ony/public.pem>
   ```
5. With `services.txt` in place, run the client:
   ```bash
   cargo run -p client
   ```
Nodes will log each routing step and the final service prints the received messages.
