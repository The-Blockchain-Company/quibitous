pub mod client;
pub mod server;

pub use client::QuibitousClient;
pub use server::QuibitousServerImpl;

mod node {
    tonic::include_proto!("tbco.chain.node"); // The string specified here must match the proto package name
}

mod types {
    tonic::include_proto!("tbco.chain.types"); // The string specified here must match the proto package name
}

mod watch {
    tonic::include_proto!("tbco.chain.watch"); // The string specified here must match the proto package name
}

use chain_core::{packer::Codec, property::DeserializeFromSlice};

pub fn read_into<T: DeserializeFromSlice>(bytes: &[u8]) -> T {
    let mut buf = Codec::new(bytes);
    T::deserialize_from_slice(&mut buf).unwrap()
}
