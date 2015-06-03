extern crate rand;
extern crate time;

pub type IdType = (u64);

pub enum PacketType {
        NewMessage = 16,
        AckMessage = 17,
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

	// data = message
	pub fn new(data: Vec<u8>, ip: String) -> Packet {
		let r = rand::random::<u64>();
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

		// version + type
		let mut v: Vec<u8> = vec![1, self.typ];
		// id
		let mut t = self.id;
		for _ in 0..8 {
			v.push(t as u8);
			t = t >> 8;
		}
		// data / payload
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
        if typ == (PacketType::NewMessage as u8) || typ == (PacketType::AckMessage as u8) {
            true
        } else {
            false
        }
    }

	pub fn deserialize(buf: *const u8, len: u32, ip: String) -> Option<Packet> {

		if len < 10 {
			return None;
		}

		let mut raw = Packet{ 
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
