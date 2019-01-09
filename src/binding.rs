use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::convert::From;

use crate::{IncomingMessage, Message, Errors, MessageType};
use crate::packet::{Packet, IdType};
use crate::xip::IpAddresses;

use std::collections::HashMap;
use std::iter::repeat;

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
			unsafe { (*target).pong(buf, len, string_from_cstr(srcip)); }
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

struct PendingPacket {
	p: Packet,
	millis: i64,
}

impl PendingPacket {
	pub fn new(p: Packet, millis: i64) -> PendingPacket {
		PendingPacket {
			p,
			millis,
		}
	}
}

pub struct SharedData {
	// Packets that have been transmitted and for which we
	// are waiting for the acknowledge.
	packets          : HashMap<u64, PendingPacket>,
}


#[repr(C)]
pub struct Network {
    tx_msg           : Sender<IncomingMessage>,
	shared           : Arc<Mutex<SharedData>>,
	status_tx        : Sender<String>,
	accept_ip: Vec<String>,
	min_siz: usize,
	max_siz: usize,
	pub current_siz: usize,
}

fn current_millis() -> i64 {
	let t = time::now().to_timespec();
	t.sec * 1000 +  (t.nsec / 1000 / 1000) as i64
}

impl Network {
	pub fn new(dev: &String, tx_msg: Sender<IncomingMessage>, status_tx: Sender<String>, accept_ip: &IpAddresses) -> Box<Network> {

		let s = Arc::new(Mutex::new(SharedData {
			packets : HashMap::new(),
		}));

		// Network must be on the heap because of the callback function.
		let mut n = Box::new(Network {
			shared: s.clone(),
            tx_msg,
			status_tx: status_tx.clone(),
			accept_ip: accept_ip.as_strings().into_iter().collect(),
			min_siz: 128,
			max_siz: 8192,
			current_siz: 8192
		});

		n.init_callback(dev);
		n.init_retry_event_receiver(s.clone());
		Network::ping(status_tx, 8192, accept_ip.as_strings().pop().unwrap());
		n
	}

	fn init_retry_event_receiver(&mut self, k: Arc<Mutex<SharedData>>) {
		thread::spawn(move || { loop {
			thread::sleep(Duration::from_millis(1000));
			let mut r = vec![];
			{
				for pp in &mut k.lock().unwrap().packets.values_mut() {
					if current_millis() > pp.millis + RETRY_TIMEOUT {
						r.push(pp.p.clone());
						pp.millis = current_millis();
					}
				}
			}
			for i in r {
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
					#[cfg(feature="debugout")]
					self.status_tx.send(String::from("[Network::init_callback] failed")).unwrap();
				},
				_ => {
					#[cfg(feature="debugout")]
					self.status_tx.send(String::from("[Network::init_callback] network initialized)")).unwrap();
				}
			}
		}
	}

	fn ping(_status_tx: Sender<String>, n: usize, ip: String) {
		//_status_tx.send(format!("probing {} {}...", ip, n)).unwrap();
		let s = format!("PROBING:{}/", n);
		let b = s.as_bytes();
		if n < b.len() {
			panic!("Invalid n.");
		}
		let v = b.iter().cloned().chain(repeat(1 as u8).take(n - s.len())).collect();
		Network::send_data_as_ping(v, ip.clone()).unwrap();
	}

	pub fn pong(&mut self, buf: *const u8, len: u32, ip: String) {

		match Packet::deserialize(buf, len, ip.clone()) {
			Some(p) => {
				if p.data.len() < 10 {
					return;
				}
				let v = "PROBING:".as_bytes();
				let (l, r) = p.data.split_at(8);
				if v == l {
					match r.iter().position(|&x| x == 47) {
						Some(pos) => {
							let s = r.iter().take(pos).cloned().collect::<Vec<u8>>();
							let n = String::from_utf8(s).unwrap();
							let c = n.parse::<usize>().unwrap();
							let b = r.iter().skip(pos + 1).take(c).all(|&x| x == 1);
							let k = r.iter().skip(pos + 1).take(c).count() + 8 + pos + 1;
							if k == c && b {
								//self.status_tx.send(format!("pong {} {} {}, increasing", len, ip, n)).unwrap();
								self.min_siz = self.current_siz;
								self.current_siz = (self.min_siz + self.max_siz) / 2;
								if self.current_siz != self.min_siz {
									Network::ping(self.status_tx.clone(), self.current_siz, ip);
								} else {
									self.status_tx.send(format!("Maximum payload size is {}.", self.current_siz)).unwrap();
								}
							} else {
								//self.status_tx.send(format!("decreasing from {}", self.current_siz));
								self.max_siz = self.current_siz;
								self.current_siz = (self.min_siz + self.max_siz) / 2;
								if self.current_siz != self.min_siz {
									Network::ping(self.status_tx.clone(), self.current_siz, ip);
								} else {
									self.status_tx.send(format!("Maximum payload size is {}.", self.current_siz)).unwrap();
								}
							}
						},
						_ => {}
					}
				}
			},
			_ => {}
		}

	}

	// This method is called with the encrypted content in buf.
	pub fn recv_packet(&mut self, buf: *const u8, len: u32, ip: String) {

		#[cfg(feature="debugout")]
		self.status_tx.send(String::from("[Network::recv_packet()] ============= called =============")).expect("send failed");

		if len == 0 {
			// TODO: hack: ip is the reason for the invalid packet
			/*
			self.status_tx.send(
				format!("[Network::recv_packet()] received invalid packet: {}", ip)).unwrap();
			*/
			return;
		}

		if self.accept_ip.iter().find(|&x| *x == ip).is_none() {
			// Ignore packet as it comes from an IP which is not accepted.
			#[cfg(feature = "show_dropped")]
			self.status_tx.send(format!("Dropped packet from {} / {:?}", ip, self.accept_ip)).expect("Send failed.");

			return;
		}

		// TODO error handling
		//self.status_tx.send(String::from("[Network::recv_packet()] receving packet")).unwrap();

		#[cfg(feature="debugout")]
		unsafe {
			let mut vv: Vec<u8> = vec![];
			for i in 0..len {
				vv.push(*buf.offset(i as isize));
			}
			self.status_tx.send(format!("[Network::recv_packet()] new message; len = {}, {:?}", len, vv)).unwrap();
		}

		let r = Packet::deserialize(buf, len, ip);
		// The payload in the packet in r is still encrypted.
		match r {
			Some(p) => {
				if p.is_file_upload() {
					self.handle_file_upload(p);
				} else if p.is_new_message() {
					#[cfg(feature="debugout")]
					self.status_tx.send(String::from("[Network::recv_packet()] new message")).unwrap();
                    self.handle_new_message(p);
                } else if p.is_ack() {
					//self.status_tx.send(String::from("[Network::recv_packet()] ack")).expect("bindings:ack failed");
                    self.handle_ack(p);
                } else {
					#[cfg(feature="debugout")]
					self.status_tx.send(String::from("[Network::recv_packet()] unknown packet type")).unwrap();
                }
			},
			None => {
				#[cfg(feature="debugout")]
				self.status_tx.send(String::from("[Network::recv_packet()] deserialization failed")).unwrap();
			}
		}
	}

    fn contains(&self, id: IdType) -> bool {

		self.shared.lock()
			.expect("Cannot lock.")
			.packets
			.contains_key(&id)
    }

	// Packet could be one of a lot of packets.
	fn handle_file_upload(&self, p: Packet) {

		if !self.contains(p.id) { // we are not the sender of the message
			let m = Message::new(p.ip.clone(), p.data.clone());

			// Send message to receiver of the last argument of Delivery::new(..., rx) which
			// is handled in Delivers::init_rx().
			match self.tx_msg.send(IncomingMessage::FileUpload(m)) {
				Err(_) => println!("handle_new_message: could not deliver message to upper layer"),
				_      => { }
			}
			Network::transmit(Packet::create_ack(p));
			// TODO error
		}
	}

	// This method is called when a new message has been received.
    fn handle_new_message(&self, p: Packet) {

        if !self.contains(p.id) { // we are not the sender of the message
            let m = Message::new(p.ip.clone(), p.data.clone());

			#[cfg(feature="debugout")]
			self.status_tx.send(format!("NEW MESSAGE: {} {}", p.data.len(), m.sha2())).unwrap();

            match self.tx_msg.send(IncomingMessage::New(m)) {
                Err(_) => println!("handle_new_message: could not deliver message to upper layer"),
                _      => { }
            }
			#[cfg(feature="debugout")]
			self.status_tx.send(String::from("binding.rs::sending ack")).expect("Could not send.");
            Network::transmit(Packet::create_ack(p));
            // TODO error
        }
    }

    fn handle_ack(&mut self, p: Packet) {
		if self.shared.lock()
			.expect("Lock failed.")
			.packets
			.remove(&p.id).is_some() {
			self.tx_msg.send(IncomingMessage::Ack(p.id)).expect("Send failed.");
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
	pub fn send_msg(msg: Message, shared: Arc<Mutex<SharedData>>, mini_id: u64) -> Result<u64, Errors> {

		let ip  = msg.get_ip();
		let buf = msg.get_payload();

		if buf.len() > MAX_MESSAGE_SIZE {
			return Err(Errors::MessageTooBig);
		}

		let p = match msg.typ {
			MessageType::FileUpload => Packet::file_upload(buf, ip, mini_id),
			_ => Packet::new(buf, ip, mini_id)
		};

		Network::wait_for_queue(shared.clone());

		// Push message before sending it. Otherwise there could be a race condition that the ACK
		// is received before message is sent.
		Network::add_packet(shared.clone(), p.clone());

		let id = p.id;
		if Network::transmit(p) {
			Ok(id)
		} else {
			Network::remove_packet(shared.clone(), id);
			Err(Errors::SendFailed)
		}
	}

	pub fn shared_data(&self) -> Arc<Mutex<SharedData>> {
		self.shared.clone()
	}

	fn remove_packet(shared: Arc<Mutex<SharedData>>, id: u64) {
		shared.lock()
			.expect("binding::push_packet: lock failed")
			.packets
			.remove(&id);
	}

	fn add_packet(shared: Arc<Mutex<SharedData>>, p: Packet) {
		shared.lock()
			.expect("binding::push_packet: lock failed")
			.packets
			.insert(p.id, PendingPacket::new(p, current_millis()));
	}

	fn queue_size(shared: Arc<Mutex<SharedData>>) -> usize {
		shared.lock()
			.expect("binding::queue_size failed")
			.packets
			.len()
	}

	fn wait_for_queue(shared: Arc<Mutex<SharedData>>) {
		while Network::queue_size(shared.clone()) > 1000 {
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

	pub fn send_data_as_ping(buf: Vec<u8>, ip: String) -> Result<u64, ()> {

		let id = rand::random::<u64>();
		let p = Packet::new(buf, ip, id);
		if Network::transmit(p) {
			Ok(id)
		} else {
			Err(())
		}
	}
}
