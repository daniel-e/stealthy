use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;

use crate::{IncomingMessage, Message, Errors, MessageType};
use crate::packet::{Packet, IdType};

const RETRY_TIMEOUT: i64      = 15000;  // TODO
const MAX_MESSAGE_SIZE: usize = (1024 * 1024 * 1024);


pub fn string_from_cstr(cstr: *const u8) -> String {

	let mut v: Vec<u8> = vec![];
	let mut i = 0;
	loop { unsafe {
		let c = *cstr.offset(i);
		if c == 0 { break; } else { v.push(c); }
		i += 1;
	}}
	String::from_utf8(v).unwrap()
}

// Callback functions.------------------------------------------------------------------

/// Callback function called by the ICMP C library.
extern "C" fn callback(target: *mut Network, buf: *const u8, len: u32, typ: u32, srcip: *const u8) {

	match typ {
		// for values look into the enum in icmp/net.h
		0 => { // ping
			unsafe { (*target).recv_packet(buf, len, string_from_cstr(srcip)); }
		},
		1 => { // pong
		},
		2 => {
			unsafe { (*target).recv_packet(buf, len, String::from("invalid length")); }
		},
		3 => {
			unsafe { (*target).recv_packet(buf, len, String::from("invalid IP length")); }
		},
		4 => {
			unsafe { (*target).recv_packet(buf, len, String::from("invalid protocol")); }
		},
		_ => { // invalid
			unsafe { (*target).recv_packet(buf, len, String::from("unknown")); }
		}
	}
}

#[link(name = "pcap")]
extern {
	fn send_icmp(ip: *const u8, buf: *const u8, siz: u16) -> libc::c_int;
}

// TODO warning about improper ctypes is disabled; we should enable it again
// and try to eliminate all warnings
#[allow(improper_ctypes)]
extern {
	fn recv_callback(
        target: *mut Network,
		dev: *const u8,
		cb: extern fn(*mut Network, *const u8, u32, u32, *const u8)) -> libc::c_int;
}

// -------------------------------------------------------------------------------------

struct SharedData {
	// Packets that have been transmitted and for which we
	// are waiting for the acknowledge.
	packets          : Vec<(Packet, i64)>,
}


#[repr(C)]
pub struct Network {
	tx               : Sender<IdType>,
    tx_msg           : Sender<IncomingMessage>,
	shared           : Arc<Mutex<SharedData>>,
	status_tx        : Sender<String>,
}

fn current_millis() -> i64 {
	let t = time::now().to_timespec();
	t.sec * 1000 +  (t.nsec / 1000 / 1000) as i64
}

impl Network {
	/// Constructs a new `Network`.
	pub fn new(dev: &String, tx_msg: Sender<IncomingMessage>, status_tx: Sender<String>) -> Box<Network> {

		let s = Arc::new(Mutex::new(SharedData {
			packets : vec![],
		}));

        // Create a channel to transport retry events.
		let (tx, rx) = channel();

		// Network must be on the heap because of the callback function.
		let mut n = Box::new(Network {
			shared: s.clone(),
            tx_msg: tx_msg,
			tx: tx,
			status_tx: status_tx
		});

		n.init_callback(dev);
		n.init_retry_event_receiver(rx, s.clone());
		n
	}

	// TODO rx is not used anymore -> remove it
	fn init_retry_event_receiver(&mut self, _rx: Receiver<IdType>, k: Arc<Mutex<SharedData>>) {
        //let tx = self.tx.clone();
		thread::spawn(move || { loop {
			thread::sleep(Duration::from_millis(1000));
			let mut r = vec![];
			{
				let mut v = k.lock().unwrap();
				for &mut (ref mut i, ref mut t) in &mut v.packets {
					if current_millis() > *t + RETRY_TIMEOUT {
						r.push(i.clone());
						*t = current_millis();
					}
				}
			}
			for i in r {
				//println!("resend");
				Network::transmit(i);
			}
		}});
	}

	fn init_callback(&mut self, dev: &String) {
		let sdev = dev.clone() + "\0";
		unsafe {
			// call to C function in icmp/net.c
			let r = recv_callback(&mut *self, sdev.as_ptr(), callback);
			match r {
				-1 => {
					self.status_tx.send(String::from("[Network::init_callback] failed")).unwrap();
				},
				_ => {
					self.status_tx.send(String::from("[Network::init_callback] network initialized)")).unwrap();
				}
			}
		}
	}

	// This method is called with the encrypted content in buf.
	pub fn recv_packet(&mut self, buf: *const u8, len: u32, ip: String) {

		if len == 0 {
			// TODO: hack: ip is the reason for the invalid packet
			/*
			self.status_tx.send(
				format!("[Network::recv_packet()] received invalid packet: {}", ip)).unwrap();
			*/
			return;
		}

		// TODO error handling
		//self.status_tx.send(String::from("[Network::recv_packet()] receving packet")).unwrap();

		let r = Packet::deserialize(buf, len, ip);
		// The payload in the packet in r is still encrypted.
		match r {
			Some(p) => {
				if p.is_file_upload() {
					self.handle_file_upload(p);
				} else if p.is_new_message() {
					self.status_tx.send(String::from("[Network::recv_packet()] new message")).unwrap();
                    self.handle_new_message(p);
                } else if p.is_ack() {
					//self.status_tx.send(String::from("[Network::recv_packet()] ack")).expect("bindings:ack failed");
                    self.handle_ack(p);
                } else {
					self.status_tx.send(String::from("[Network::recv_packet()] unknown packet type")).unwrap();
                }
			},
			None => {
				self.status_tx.send(String::from("[Network::recv_packet()] deserialization failed")).unwrap();
			}
		}
	}

    fn contains(&self, id: IdType) -> bool {

        let shared = self.shared.clone();
        let v = shared.lock().expect("binding::contains: lock failes");
        for &(ref i, _t) in &v.packets {
            if i.id == id {
                return true;
            }
        }
        false
    }

	// Packet could be one of a lot of packets.
	fn handle_file_upload(&self, p: Packet) {

		//println!("TTT upload");
		if !self.contains(p.id) { // we are not the sender of the message
			let m = Message::new(p.ip.clone(), p.data.clone());

			// Send message to receiver of the last argument of Delivery::new(..., rx) which
			// is handled in Delivers::init_rx().
			match self.tx_msg.send(IncomingMessage::FileUpload(m)) {
				Err(_) => println!("handle_new_message: could not deliver message to upper layer"),
				_      => { }
			}
			//println!("TTT upload ACK");
			Network::transmit(Packet::create_ack(p));
			// TODO error
		}
	}

    fn handle_new_message(&self, p: Packet) {

        if !self.contains(p.id) { // we are not the sender of the message
            let m = Message::new(p.ip.clone(), p.data.clone());
            match self.tx_msg.send(IncomingMessage::New(m)) {
                Err(_) => println!("handle_new_message: could not deliver message to upper layer"),
                _      => { }
            }
            Network::transmit(Packet::create_ack(p));
            // TODO error
        }
    }

    fn handle_ack(&mut self, p: Packet) {

		//println!("TTT handle_ack");
        let shared = self.shared.clone();
        let mut v = shared.lock().expect("binding::handle_ack: lock failed");
        let mut c = 0;
        let mut b: bool = false;

        for &(ref i, _t) in &v.packets { // search the id
            if i.id == p.id {
                b = true;
                break;
            }
            c += 1;
        }
        if b {
			//println!("TTT removing");
            v.packets.swap_remove(c);
            match self.tx_msg.send(IncomingMessage::Ack(p.id)) {
                Err(_) => println!("handle_ack: could not deliver ack to upper layer"),
                _      => { }
            }
        }
  }

	/// message format:
	/// u8 : version { 1 }
	/// u8 : type    { 16 = send message, 17 = ack }
	/// u64: id
	/// Vec<u8> : payload (msg) from layer above  (if type == 1)

	/// Sends a message to the receiver ip.
	///
	/// The message is send via an ICMP echo request and the function
	/// returns to the caller a handle which can be used by the caller
	/// to identify the message. The message is now in the status
	/// `transmitting`. As soon as an acknowledge is received the
	/// configured callback function is called with the handle.
	///
	/// ip  = IPv4 of the receiver
	/// buf = data to be transmitted to the receiver
	pub fn send_msg(&self, msg: Message) -> Result<u64, Errors> {

		let ip  = msg.get_ip();
		let buf = msg.get_payload();

		if buf.len() > MAX_MESSAGE_SIZE {
			return Err(Errors::MessageTooBig);
		}

		let p = match msg.typ {
			MessageType::FileUpload => Packet::file_upload(buf, ip),
			_ => Packet::new(buf, ip)
		};

		self.wait_for_queue();

		// Push message before sending it. Otherwise there could be a race condition that the ACK
		// is received before message is sent.
		self.push_packet(p.clone());

		if Network::transmit(p.clone()) {
			Ok(p.id)
		} else {
			self.pop_packet();
			Err(Errors::SendFailed)
		}
	}

	fn pop_packet(&self) {
		self.shared.lock().expect("binding::push_packet: lock failed").packets.pop();
	}

	fn push_packet(&self, p: Packet) {
		self.shared.lock().expect("binding::push_packet: lock failed").packets.push((p, current_millis()));
	}

	fn queue_size(&self) -> usize {
		self.shared.lock().expect("binding::queue_size failed").packets.len()
	}

	fn wait_for_queue(&self) {
		while self.queue_size() >= 1000 {
			thread::sleep(Duration::from_millis(50));
		}
	}

	fn transmit(p: Packet) -> bool {
		let v = p.serialize();
		let ip = p.ip.clone() + "\0";
		unsafe {
			send_icmp(ip.as_ptr(), v.as_ptr(), v.len() as u16) == 0
		}
	}
}
