mod binding;
mod blowfish;
mod crypto;
mod delivery;
mod packet;
mod rsa;
mod tools;

use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};

use crypto::Encryption;
use binding::Network;


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
	pub fn new(ip: &String, buf: Vec<u8>) -> Message {
		Message {
			ip : ip.clone(),
			buf: buf,
            typ: MessageType::NewMessage,
		}
	}

	pub fn ack(ip: &String) -> Message {
		Message {
			ip : ip.clone(),
			buf: vec![],
            typ: MessageType::AckMessage,
		}
	}
}




pub struct Layers {
    encryption_layer: Arc<Encryption>,
    network_layer   : Box<Network>,
}

impl Layers {
    pub fn default(key: &String, device: &String) -> (Receiver<Message>, Layers) {

        let (tx, rx) = channel::<Message>();
        let e = Encryption::new(&key);
        let (ltx, lrx) = channel(); // connection between network and layers
        let n = Network::new(device.clone(), ltx);
        (rx, Layers::new(e, n, lrx, tx))
    }

    fn new(
        e              : Encryption, 
        n              : Box<Network>,
        rx_from_network: Receiver<Message>,
        tx_application : Sender<Message>) -> Layers {

        let enc = Arc::new(e);
        let enc_thread = enc.clone();

        // thread to handle received messages vi rx_from_network
        thread::spawn(move || {
            loop { match rx_from_network.recv() {
                Ok(msg) => {
                    match msg.typ {
                        MessageType::NewMessage => { 
                            match enc_thread.decrypt(msg.buf) {
                                Some(buf) => {
                                    match tx_application.send(Message::new(&msg.ip, buf)) {
                                        Err(_) => println!("error: could not deliver new message"),
                                        _      => { }
                                    }
                                }

                                None => { println!("{} error: could not decode message", msg.ip) }  // TODO error handling
                            }
                        }
                        MessageType::AckMessage => { 
                            match tx_application.send(msg) {
                                Err(_) => println!("error: could not deliver ack"),
                                _      => { }
                            }
                        }
                    }
                }
                Err(_) => { println!("Failed to receive message."); }
            }};
        });

        Layers {
            encryption_layer: enc,
            network_layer   : n,
        }
    }

    pub fn send(&mut self, msg: Message) -> Result<u64, Errors> {

        let e = self.encryption_layer.encrypt(msg.buf);
        let m = Message::new(&msg.ip, e); // TODO is this always a new message?

        self.network_layer.send_msg(m)
    }
}



