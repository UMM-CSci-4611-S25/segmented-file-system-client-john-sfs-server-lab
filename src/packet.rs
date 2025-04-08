#[derive(Debug)]
pub enum PacketType {
    HeaderPacket(Header),
    DataPacket(Data)
}

#[derive(Debug)]
pub enum PacketError{
    EmptyPacket(String),
    PacketOverflow(String),
}

#[derive(Debug)]
pub struct Header {
    pub file_name: [u8; 1024]
}

#[derive(Debug)]
pub struct Data {
    packet_number: u16,
    last_packet: bool,
    data: [u8; 1024]
}

impl Data {
    pub fn get_data(&self) -> &[u8; 1024] {
        &self.data
    }

    pub fn packet_num(&self) -> u16 {
        self.packet_number
    }
    pub fn is_last(&self) -> bool {
        self.last_packet
    }
}

#[derive(Debug)]
pub struct Packet {
    file_id: u8,
    packet_type: PacketType
}

//Implements a get and set for the file_id.

impl Packet {
    pub fn file_id(&self) -> u8 {
        self.file_id
    }

    pub fn set_file_id(&mut self, new: u8) {
        self.file_id = new;
    }

    pub fn is_header(&self) -> bool {
        match self.packet_type {
            PacketType::HeaderPacket(_) => true,
            PacketType::DataPacket(_) => false
        }
    }

    pub fn get_contents(self) -> PacketType {
        self.packet_type
    }
}

//Implement from<[u8; ...]> -> Packet
impl TryFrom<&[u8]> for Packet {
    type Error = PacketError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
       if value.len() < 2 {
        return Err(PacketError::EmptyPacket("Missing required bytes in packet.".to_string()));
       }
       if value.len() > 1028 {
        return Err(PacketError::PacketOverflow("Too many bytes in packet, must be less than 1028 bytes.".to_string()));
       }
       let status_byte: u8 = value[0];
       let file_id: u8 = value[1];

       let is_header: bool = status_byte & 1 == 0;
       if is_header {
        let head: Header = Header {
                file_name: value[2..].try_into().expect("Could not coerce into array.")
            };
            Ok(Packet{file_id, packet_type: PacketType::HeaderPacket(head)})
        } else {
            let packet_number: u16 = u16::from_be_bytes([value[2], value[3]]);
            let last_packet: bool = status_byte % 4 == 3;
            let data: Data = Data { 
                packet_number, 
                last_packet, 
                data: value[4..].try_into().expect("Could not convert from slice to array.") 
            };
            Ok(Packet { file_id, packet_type: PacketType::DataPacket(data) })
        }
    }
}