pub mod packet;
pub mod file_manager;

use crate::packet::{Packet,PacketError};
use crate::file_manager::FileGroup;
// Below is a version of the `main` function and some error types. This assumes
// the existence of types like `FileManager`, `Packet`, and `PacketParseError`.
// You can use this code as a starting point for the exercise, or you can
// delete it and write your own code with the same function signature.

use std::{
    io::{self, Write},
    net::UdpSocket,
};


#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    PacketError(PacketError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::IoError(e)
    }
}

impl From<PacketError> for ClientError {
    fn from(e: PacketError) -> Self {
        Self::PacketError(e)
    }
}

fn main() -> Result<(), ClientError> {
    let sock = UdpSocket::bind("0.0.0.0:7077")?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr)?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]);

    let mut file_group = FileGroup::default();

    while !file_group.received_all_packets() {
        let len = sock.recv(&mut buf)?;
        let packet: Packet = buf[..len].try_into()?;
        print!(".");
        io::stdout().flush()?;
        file_group.process_packet(packet);
    }

    file_group.write_all_files()?;

    Ok(())
}
