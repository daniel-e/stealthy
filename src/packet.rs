//extern crate rand;
//extern crate time;

pub type IdType = u64;

pub enum PacketType {
    NewMessage = 16,
    AckMessage = 17,
	FileUpload = 18,
	HelloMessage = 19,
}

pub struct Packet {
	// The id of the packet that is transmitted. It is used to identify
	// the ack for that message.
	pub id:      IdType,
	pub data:    Vec<u8>,      // data packet from the caller
	pub created: time::PreciseTime,
	pub ip:      String,
    pub typ:     u8,
}

impl Packet {

    pub fn is_new_message(&self) -> bool {
        self.typ == (PacketType::NewMessage as u8)
    }

    pub fn is_ack(&self) -> bool {
        self.typ == (PacketType::AckMessage as u8)
    }

	pub fn is_file_upload(&self) -> bool {
		self.typ == (PacketType::FileUpload as u8)
	}

	pub fn is_hello(&self) -> bool {
		self.typ == (PacketType::HelloMessage as u8)
	}

	pub fn hello(data: Vec<u8>, ip: String, r: u64) -> Packet {
		Packet {
			data: data,
			id: r,
			created: time::PreciseTime::now(),
			ip: ip,
			typ: PacketType::HelloMessage as u8,
		}
	}

	pub fn file_upload(data: Vec<u8>, ip: String, r: u64) -> Packet {
		Packet {
			data: data,
			id: r,
			created: time::PreciseTime::now(),
			ip: ip,
			typ: PacketType::FileUpload as u8,
		}
	}

	// data = message
	pub fn new(data: Vec<u8>, ip: String, r: u64) -> Packet {
		Packet {
			data: data, 
			id: r,
			created: time::PreciseTime::now(),
			ip: ip,
            typ: PacketType::NewMessage as u8,
		}
	}

	pub fn clone(&self) -> Packet {
		Packet {
			id: self.id,
			data: self.data.clone(),
			created: self.created.clone(),
			ip: self.ip.clone(),
            typ: self.typ,
		}
	}

	pub fn serialize(&self) -> Vec<u8> {

		// if you change someting check delivery::send_msg

		// version + type
		let mut v: Vec<u8> = vec![1, self.typ];         // 2B
		// id
		let mut t = self.id;
		for _ in 0..8 {                                // 8B
			v.push(t as u8);
			t = t >> 8;
		}
		// data / payload                              // data
		for k in self.data.clone() {
			v.push(k);
		}
		v
	}

    pub fn create_ack(p: Packet) -> Packet {

        Packet {
            id: p.id,
            data: vec![],
            created: time::PreciseTime::now(),
            ip: p.ip,
            typ: PacketType::AckMessage as u8,
        }
  }

    fn valid_type(typ: u8) -> bool {
		typ == (PacketType::NewMessage as u8) ||
			typ == (PacketType::AckMessage as u8) ||
			typ == (PacketType::FileUpload as u8) ||
			typ == (PacketType::HelloMessage as u8)
    }

	pub fn deserialize(buf: *const u8, len: u32, ip: String) -> Option<Packet> {

		if len < 10 {
			return None;
		}

		let mut raw = Packet {
			id: 0, 
			data: vec![], 
			created: time::PreciseTime::now(),
			ip: ip,
            typ: 0
		};

		unsafe {
			let ver : u8 = *buf.offset(0);
			let typ : u8 = *buf.offset(1);

			if ver != 1 || !Packet::valid_type(typ) {
				return None;
			}
			for i in 0..8 {
				raw.id = (raw.id << 8) + (*buf.offset(2 + 7 - i) as u64);
			}
			for i in 10..len {
				raw.data.push(*buf.offset(i as isize));
			}
            raw.typ = typ;
			Some(raw)
		}
	}
}
