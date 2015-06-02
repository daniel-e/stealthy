//extern crate log;
extern crate time;
extern crate libc;

mod packet;
mod tools;
pub mod delivery;

use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

pub enum MessageType {
    NewMessage,
    AckMessage
}

pub enum Errors {
	MessageTooBig,
	SendFailed
}

pub struct Message {
	pub ip : String,
	pub buf: Vec<u8>,
    pub typ: MessageType,
}

impl Message {
	pub fn new(ip: String, buf: Vec<u8>, typ: MessageType) -> Message {
		Message {
			ip : ip,
			buf: buf,
            typ: typ,
		}
	}
}

const RETRY_TIMEOUT: u32      = 15000;
const MAX_MESSAGE_SIZE: usize = (10 * 1024);


// Callback functions.------------------------------------------------------------------

/// Callback function called by the ICMP C library.
extern "C" fn callback(target: *mut Network, buf: *const u8, len: u32, typ: u32, srcip: *const u8) {

	if typ == 0 { // check only ping messages
		unsafe { (*target).recv_packet(buf, len, tools::string_from_cstr(srcip)); }
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
    tx_msg           : Sender<Message>,
	shared           : Arc<Mutex<SharedData>>,
}

impl Network {
	/// Constructs a new `Network`.
	pub fn new(dev: String, tx_msg: Sender<Message>) -> Box<Network> {

		let s = Arc::new(Mutex::new(SharedData {
			packets : vec![],
		}));

        // Create a channel to transport retry events.
		let (tx, rx) = channel();

		// Network must be on the heap because of the callback function.
		let mut n = Box::new(Network {
			shared: s.clone(),
            tx_msg: tx_msg,
//			new_msg_cb: new_msg_cb,
  //          ack_msg_cb: ack_msg_cb,
			tx: tx,
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

	fn init_callback(&mut self, dev: String) {
		let sdev = dev.clone() + "\0";
		unsafe {
			recv_callback(&mut *self, sdev.as_ptr(), callback); // TODO error handling
		}
	}

	pub fn recv_packet(&mut self, buf: *const u8, len: u32, ip: String) {
		let r = packet::Packet::deserialize(buf, len, ip);
		match r {
			Some(p) => {
                if p.is_new_message() {
                    self.handle_new_message(p);
                } else if p.is_ack() {
                    self.handle_ack(p);
                } else {
                    println!("recv_packet: unknown packet type");
                }
			},
			None => { println!("recv_packet: could not deserialize received packet"); }
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
   			let m = Message {
				ip : p.ip.clone(),
				buf: p.data.clone(),
                typ: MessageType::NewMessage,
			};
            self.tx_msg.send(m);
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

            let m = Message {
                ip : p.ip.clone(),
                buf: vec![],
                typ: MessageType::AckMessage
            };
            self.tx_msg.send(m); // TODO send id of ack
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
	pub fn send_msg(&mut self, msg: Message) -> Result<u64, Errors> {

		let ip  = msg.ip.clone();
		let buf = msg.buf.clone();

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
			thread::sleep_ms(RETRY_TIMEOUT);
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





