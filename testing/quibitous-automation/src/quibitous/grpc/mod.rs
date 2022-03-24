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

use chain_core::mempack::{ReadBuf, Readable};

pub fn read_into<T: Readable>(bytes: &[u8]) -> T {
    let mut buf = ReadBuf::from(bytes);
    T::read(&mut buf).unwrap()
}
