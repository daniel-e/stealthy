extern crate libc;

use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;

//use std::fs::{File, OpenOptions};
//use std::io::Write;

use ::packet;
use ::misc::{IncomingMessage, Message, Errors};

const RETRY_TIMEOUT: u64      = 15000;
const MAX_MESSAGE_SIZE: usize = (10 * 1024);


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

	// let mut f = OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log").unwrap();
	// f.write_fmt(format_args!("binding::callback()\n")).unwrap();

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
	packets          : Vec<packet::Packet>,
}


#[repr(C)]
pub struct Network {
	tx               : Sender<packet::IdType>,
    tx_msg           : Sender<IncomingMessage>,
	shared           : Arc<Mutex<SharedData>>,
	status_tx        : Sender<String>,
}

impl Network {
	/// Constructs a new `Network`.
	pub fn new(dev: &String, tx_msg: Sender<IncomingMessage>, status_tx: Sender<String>) -> Box<Network> {

		// let mut f = OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log").unwrap();
		// f.write_fmt(format_args!("Network::new()\n")).unwrap();

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

	fn init_retry_event_receiver(&self, rx: Receiver<packet::IdType>, k: Arc<Mutex<SharedData>>) {
        let tx = self.tx.clone();
		thread::spawn(move || { loop { match rx.recv() {
			Ok(id) => {
				let v = k.lock().unwrap();
                for i in &v.packets {
                    if i.id == id {
                        Network::transmit(i.clone());
                        Network::init_retry(tx.clone(), id);
                    }
                }
			}
			_ => { println!("error in receiving"); }
		}}});
	}

	fn init_callback(&mut self, dev: &String) {
		let sdev = dev.clone() + "\0";
		unsafe {
			// call to C function in icmp/net.c
			let r = recv_callback(&mut *self, sdev.as_ptr(), callback);
			match r {
				-1 => {
					// let mut f = OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log").unwrap();
					// f.write_fmt(format_args!("A\n")).unwrap();

					self.status_tx.send(String::from("[Network::init_callback] failed")).unwrap();
				},
				_ => {
					// let mut f = OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log").unwrap();
					// f.write_fmt(format_args!("B\n")).unwrap();

					self.status_tx.send(String::from("[Network::init_callback] network initialized)")).unwrap();
				}
			}
		}
	}

	pub fn recv_packet(&mut self, buf: *const u8, len: u32, ip: String) {

		// let mut f = OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log").unwrap();
		// f.write_fmt(format_args!("bla\n")).unwrap();

		if len == 0 {
			// TODO: hack: ip is the reason for the invalid packet
			/*
			self.status_tx.send(
				format!("[Network::recv_packet()] received invalid packet: {}", ip)).unwrap();
			*/
			return;
		}

		// TODO error handling
		self.status_tx.send(String::from("[Network::recv_packet()] receving packet")).unwrap();

		// let mut f = OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log").unwrap();
		// f.write_fmt(format_args!("bla\n")).unwrap();

		let r = packet::Packet::deserialize(buf, len, ip);
		match r {
			Some(p) => {
                if p.is_new_message() {
					self.status_tx.send(String::from("[Network::recv_packet()] new message")).unwrap();
                    self.handle_new_message(p);
                } else if p.is_ack() {
					self.status_tx.send(String::from("[Network::recv_packet()] ack")).unwrap();
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

    fn contains(&self, id: packet::IdType) -> bool {

        let shared = self.shared.clone();
        let v = shared.lock().unwrap();
        for i in &v.packets {
            if i.id == id {
                return true;
            }
        }
        false
    }

    fn handle_new_message(&self, p: packet::Packet) {

        if !self.contains(p.id) { // we are not the sender of the message

            let m = Message::new(p.ip.clone(), p.data.clone());
            match self.tx_msg.send(IncomingMessage::New(m)) {
                Err(_) => println!("handle_new_message: could not deliver message to upper layer"),
                _      => { }
            }
            Network::transmit(packet::Packet::create_ack(p));
            // TODO error
        }
    }

    fn handle_ack(&mut self, p: packet::Packet) {

        let shared = self.shared.clone();
        let mut v = shared.lock().unwrap();
        let mut c = 0;
        let mut b: bool = false;

        for i in &v.packets {
            if i.id == p.id {
                b = true;
                break;
            }
            c += 1;
        }
        if b {
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
			Err(Errors::MessageTooBig)
		} else {
			let p = packet::Packet::new(buf.clone(), ip.clone());

			// We push the message before we send the message in case that
			// the callback for ack is called before the message is in the
			// queue.
			let v = self.shared.clone();
			let mut k = v.lock().unwrap();
			k.packets.push(p.clone());

			if Network::transmit(p.clone()) {
				Network::init_retry(self.tx.clone(), p.id);
				Ok(p.id)
			} else {
				k.packets.pop();
				Err(Errors::SendFailed)
			}
		}
	}

	fn init_retry(tx: Sender<u64>, id: packet::IdType) {

		thread::spawn(move || {
			thread::sleep(Duration::from_millis(RETRY_TIMEOUT));
			match tx.send(id) {
				Err(_) => { println!("init_retry: sending event through channel failed"); }
				_ => { }
			}
		});
	}

	fn transmit(p: packet::Packet) -> bool {

		let v  = p.serialize();
		let ip = p.ip.clone() + "\0";
		unsafe {
			send_icmp(ip.as_ptr(), v.as_ptr(), v.len() as u16) == 0
		}
	}
}
