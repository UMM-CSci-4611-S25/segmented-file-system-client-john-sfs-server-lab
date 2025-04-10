use std::ffi::OsString;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::ffi::OsStringExt;

use crate::packet::{Header, Data, Packet, PacketType};
use crate::ClientError;

#[derive(Debug)]
#[derive(Clone)]
struct FileData {
    file_name: Option<OsString>,
    packet_count: Option<usize>,
    packets: HashMap<u16, Vec<u8>>, 
}


impl FileData {
    pub fn has_all_packets(&self) -> bool {
        if self.packet_count.is_none() {
            false
        }
        else {
            let mut i: usize = 0;
            while i < self.packet_count.unwrap() {
                if !self.packets.contains_key(&(i as u16)) {
                    return false;
                }
                i+=1; 
            }
            true
        }
    }

    pub fn write_file(&self) -> std::io::Result<()> {
        let mut file = File::create(self.file_name.as_ref().unwrap())?;
        for i in 1..self.packet_count.unwrap() {
            let out_byte = self.packets.get(&(i as u16)).unwrap();
            file.write_all(out_byte)?;
        } 
        Ok(())
    }

    pub fn update_data_packet(&mut self, key: u16, contents: Vec<u8>) {
        self.packets.insert(key, contents);
    }
}

#[derive(Default)]
pub struct FileGroup {
    files: HashMap<u8, FileData>
}

impl FileGroup {
    pub fn process_packet(&mut self, packet: Packet) {
        let f_id = packet.file_id();
        let header_packet: bool = packet.is_header();
        let pt: PacketType = packet.get_contents();
        if header_packet {
            //This extracts the header data from the PacketType() enum, 
            //since it's already known that the packet type will be header.
            //Therefore, we ignore the DataPacket branch.
            let data: Header = match pt {
                PacketType::HeaderPacket(h) => Some(h),
                PacketType::DataPacket(_) => None,
            }.unwrap();

            let target_file = self.files.entry(f_id).or_insert(FileData { 
                    file_name: Some(OsString::from_vec(data.file_name.to_vec())), 
                    packet_count: None, 
                    packets: HashMap::new() 
                });
            target_file.file_name = Some(OsString::from_vec(data.file_name.to_vec()));
        }
        else {
            //This means we're processing a data packet.
            let data: Data = match pt {
                PacketType::HeaderPacket(_) => None,
                PacketType::DataPacket(d) => Some(d),
            }.unwrap();
           
            let target_file = self.get_mut_file_from_key(f_id).expect("");
            target_file.update_data_packet(data.packet_num(), data.get_data().into());
        }
    }

    fn get_mut_file_from_key(&mut self, key: u8) -> Option<&mut FileData> {
        self.files.get_mut(&key)
    }

    pub fn received_all_packets(&self) -> bool {
        if self.files.len() < 3 {
            return false;
        }
        for file in &self.files {
            if !file.1.has_all_packets() {
                return false;
            }
        }
        true
    }

    pub fn write_all_files(&self) -> Result<(), ClientError> { 
        for file in &self.files {
            file.1.write_file()?;
        }
        Ok(())
    }
}