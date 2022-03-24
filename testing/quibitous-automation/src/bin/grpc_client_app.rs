use quibitous_automation::quibitous::grpc::client::QuibitousClient;
use rand::Rng;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client = QuibitousClient::from_address(&format!("{}:{}", args[1], args[2])).unwrap();
    let mut auth_nonce = [0u8; 32];
    rand::thread_rng().fill(&mut auth_nonce[..]);
    let response = client.handshake(&auth_nonce);
    println!("{:?}", response);
}
