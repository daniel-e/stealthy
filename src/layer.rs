use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::cryp::{Encryption, SymmetricEncryption, AsymmetricEncryption};  // Implemenation for encryption layer
use crate::delivery::Delivery;
use crate::binding::Network;
use crate::types::{ErrorType, IncomingMessage, Message, MessageType};
use crate::iptools::IpAddresses;

pub struct Layer {
    pub rx    : Receiver<IncomingMessage>,
    pub layers: Layers,
}

pub struct Layers {
    encryption_layer: Arc<Box<Encryption>>,
    delivery_layer  : Arc<Box<Delivery>>,
    status_tx       : Sender<String>,
}

impl Layers {

    pub fn symmetric(hexkey: &String, device: &String, status_tx: Sender<String>, accept_ip: &IpAddresses) -> Result<Layer, &'static str> {

        Layers::init(Box::new(SymmetricEncryption::new(hexkey)?), device, status_tx, accept_ip)
    }

    pub fn asymmetric(pubkey_file: &String, privkey_file: &String, device: &String, status_tx: Sender<String>, accept_ip: &IpAddresses) -> Result<Layer, &'static str> {

        Layers::init(Box::new(
            AsymmetricEncryption::new(&pubkey_file, &privkey_file)?
        ), device, status_tx, accept_ip
        )
    }

    pub fn send(&self, msg: Message, id: u64, background: bool) {

        let s = self.status_tx.clone();
        let e = self.encryption_layer.clone();
        let p = self.delivery_layer.get_pending();
        let shared = self.delivery_layer.get_shared();
        let n = self.delivery_layer.max_size();

        let t = thread::spawn(move || {
            match e.encrypt(&msg.buf) {
                Ok(buf) => {
                    Delivery::send_msg(msg.set_payload(buf), id, p, shared, s.clone(), n).run();
                },
                _ => {
                    s.send(format!("Encryption failed.")).expect("Send failed.");
                }
            }
        });

        if !background {
            t.join().expect("Join failed.");
        }
    }

    pub fn encryption_key(&self) -> Vec<u8> {
        self.encryption_layer.encryption_key()
    }

    // ------ private functions

    fn init(e: Box<Encryption>, device: &String, status_tx: Sender<String>, accept_ip: &IpAddresses) -> Result<Layer, &'static str> {

        // network  tx1 --- incoming message ---> rx1 delivery
        // delivery tx2 --- incoming message ---> rx2 layers
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        Ok(Layers::new(e,
                       Delivery::new(
                           Network::new(device, tx1, status_tx.clone(), accept_ip),
                           tx2,
                           rx1,
                           status_tx.clone(),
                       ),
                       rx2,
                       status_tx
        ))
    }

    fn new(e: Box<Encryption>, d: Delivery, rx_network: Receiver<IncomingMessage>, status_tx: Sender<String>) -> Layer {

        // tx is used to send received messages to the application via rx
        let (tx, rx) = channel::<IncomingMessage>();

        let l = Layers {
            encryption_layer: Arc::new(e),
            delivery_layer: Arc::new(Box::new(d)),
            status_tx: status_tx.clone()
        };

        l.recv_loop(tx, rx_network);
        Layer {
            rx: rx,
            layers: l,
        }
    }

    /// Listens for incoming messages and processes them.
    fn recv_loop(&self, tx: Sender<IncomingMessage>, rx: Receiver<IncomingMessage>) {

        let enc = self.encryption_layer.clone();
        let status_tx = self.status_tx.clone();

        thread::spawn(move || { loop { match rx.recv() {
            Ok(msg) => match Layers::handle_message(msg, enc.clone(), status_tx.clone()) {
                Some(m) => match tx.send(m) {
                    Err(_) => panic!("Channel closed."),
                    _ => { }
                },
                _ => Layers::err(ErrorType::DecryptionError, "Could not decrypt received message.", &tx)
            },
            _ => Layers::err(ErrorType::ReceiveError, "Could not receive message.", &tx)
        }}});
    }

    /// Notifies the application about an error.
    fn err(e: ErrorType, msg: &str, tx: &Sender<IncomingMessage>) {

        match tx.send(IncomingMessage::Error(e, msg.to_string())) {
            Ok(_) => { }
            // If the receiver has hung up quit the application.
            _ => panic!("Channel closed.")
        }
    }

    /// Decrypts incoming messages of type "new" or returns the message without
    /// modification if it is not of type "new".
    fn handle_message(m: IncomingMessage, enc: Arc<Box<Encryption>>, _status_tx: Sender<String>) -> Option<IncomingMessage> {

        // TODO error handling
        #[cfg(feature="debugout")]
            _status_tx.send(String::from("[Layers::handle_message()] decrypting message")).unwrap();

        match m {
            IncomingMessage::New(msg) => {
                #[cfg(feature="debugout")]
                    _status_tx.send(format!("[Layers::handle_message()] new message {}", msg.buf.len())).unwrap();

                match enc.decrypt(&msg.buf) {
                    Ok(buf) => Some(IncomingMessage::New(msg.set_payload(buf))),
                    Err(_m) => {
                        #[cfg(feature="debugout")]
                            _status_tx.send(format!("[Layers::handle_message()] decrypt returned with error. {}", _m)).unwrap();
                        None
                    }
                }
            },
            IncomingMessage::FileUpload(msg) => {
                match enc.decrypt(&msg.buf) {
                    Ok(buf) => Some(IncomingMessage::FileUpload(msg.set_payload(buf))),
                    _ => {
                        println!("decryption failed");
                        None
                    }
                }
            },
            IncomingMessage::Ack(_) => Some(m),
            IncomingMessage::Error(_, _) => Some(m),
            IncomingMessage::AckProgress(_, _, _) => Some(m)
        }
    }
}
