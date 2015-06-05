mod binding;
mod blowfish;
mod crypto;
mod delivery;
mod packet;
mod rsa;
mod tools;

use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crypto::Encryption;  // Implemenation for encryption layer
use binding::Network;    // Implemenation for network layer


pub enum MessageType {
    NewMessage,
    AckMessage
}

impl Clone for MessageType {
    fn clone(&self) -> MessageType { 
        match *self {
            MessageType::NewMessage => MessageType::NewMessage,
            MessageType::AckMessage => MessageType::AckMessage
        }
    }
}


pub enum Errors {
	MessageTooBig,
	SendFailed
}


pub struct Message {
    /// Contains the destination ip for outgoing messages, source ip from incoming messages.
	ip : String,
    typ: MessageType,
	buf: Vec<u8>,
}


impl Message {
	pub fn new(ip: String, buf: Vec<u8>) -> Message { Message::create(ip, buf, MessageType::NewMessage) }

	pub fn ack(ip: String) -> Message { Message::create(ip, vec![], MessageType::AckMessage) }

    pub fn set_payload(&self, buf: Vec<u8>) -> Message { 
        Message::create(self.get_ip(), buf, self.get_type())
    }

    pub fn get_payload(&self) -> Vec<u8> { self.buf.clone() }

    /// Returns the destination ip for outgoing messages or the source ip from incoming messages.
    pub fn get_ip(&self) -> String { self.ip.clone() }

    pub fn get_type(&self) -> MessageType { self.typ.clone() }

    fn create(ip: String, buf: Vec<u8>, typ: MessageType) -> Message {
		Message {
			ip: ip,
			buf: buf,
            typ: typ,
		}
	}
}


pub struct Layers {
    encryption_layer: Arc<Encryption>,
    network_layer   : Box<Network>
}


impl Layers {
    pub fn default(key: &String, device: &String) -> (Receiver<Message>, Layers) {

        // channel between network and this struct
        let (tx, rx) = channel();

        Layers::new(
            Encryption::new(key),
            Network::new(device, tx),
            rx
        )
    }

    pub fn send(&self, msg: Message) -> Result<u64, Errors> {

        let m = msg.set_payload(self.encryption_layer.encrypt(&msg.buf));
        self.network_layer.send_msg(m)
    }

    fn new(e: Encryption, n: Box<Network>, rx_network: Receiver<Message>) -> (Receiver<Message>, Layers) {

        // channel between application and this struct
        let (tx, rx) = channel::<Message>();

        let l = Layers {
                    encryption_layer: Arc::new(e),
                    network_layer: n
                };

        l.spawn_receiver(tx.clone(), rx_network);
        (rx, l)
    }

    fn spawn_receiver(&self, tx: Sender<Message>, rx: Receiver<Message>) {

        let enc = self.encryption_layer.clone();

        thread::spawn(move || { loop { match rx.recv() {
            Ok(msg) => {
                match Layers::handle_message(msg, enc.clone()) {
                    Some(msg) => { 
                        match tx.send(msg) {
                            Err(_) => { println!("error: could not deliver received message to application"); }
                            _ => { }
                        }
                    }
                    _ => { println!("error: could not handle received message") }
                }
            }
            _ => { println!("error: failed to receive message"); }
        }}});
    }

    fn handle_message(m: Message, enc: Arc<Encryption>) -> Option<Message> {

        match m.typ {
            MessageType::NewMessage => { 
                let buf = enc.decrypt(m.buf.clone());
                match buf {
                    Some(buf) => { Some(m.set_payload(buf)) }
                    None      => { None }
                }
            }

            MessageType::AckMessage => { Some(m) }
        }
    }
}



