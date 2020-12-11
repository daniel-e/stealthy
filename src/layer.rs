use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::cryp::{Encryption, SymmetricEncryption, AsymmetricEncryption};  // Implemenation for encryption layer
use crate::delivery::Delivery;
use crate::binding::Network;
use crate::message::{IncomingMessage, Message};
use crate::error::ErrorType;
use crate::iptools::IpAddresses;
use crate::{Console, Ips};

pub struct Layer {
    pub rx    : Receiver<IncomingMessage>,
    pub layers: Layers,
}

pub struct Layers {
    encryption_layer: Arc<Box<dyn Encryption>>,
    delivery_layer  : Arc<Box<Delivery>>,
    console: Console,
}

impl Layers {

    pub fn symmetric(hexkey: &String, device: &String, console: Console, accept_ip: Ips) -> Result<Layer, &'static str> {

        Layers::init(Box::new(SymmetricEncryption::new(hexkey)?), device, console, accept_ip)
    }

    pub fn asymmetric(pubkey_file: &String, privkey_file: &String, device: &String, console: Console, accept_ip: Ips) -> Result<Layer, &'static str> {

        Layers::init(Box::new(
            AsymmetricEncryption::new(&pubkey_file, &privkey_file)?
        ), device, console, accept_ip
        )
    }

    pub fn send(&self, msg: Message, id: u64, background: bool) {

        let console = self.console.clone();
        let e = self.encryption_layer.clone();
        let p = self.delivery_layer.get_pending();
        let shared = self.delivery_layer.get_shared();
        let n = self.delivery_layer.max_size();

        let t = thread::spawn(move || {
            match e.encrypt(&msg.buf) {
                Ok(buf) => {
                    Delivery::send_msg(msg.set_payload(buf), id, p, shared, console.clone(), n).run();
                },
                _ => {
                    console.status(format!("Encryption failed."));
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

    fn init(e: Box<dyn Encryption>, device: &String, console: Console, accept_ip: Ips) -> Result<Layer, &'static str> {

        // network  tx1 --- incoming message ---> rx1 delivery
        // delivery tx2 --- incoming message ---> rx2 layers
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        Ok(Layers::new(e,
                       Delivery::new(
                           Network::new(device, tx1, console.clone(), accept_ip),
                           tx2,
                           rx1,
                           console.clone(),
                       ),
                       rx2,
                       console
        ))
    }

    fn new(e: Box<dyn Encryption>, d: Delivery, rx_network: Receiver<IncomingMessage>, console: Console) -> Layer {

        // tx is used to send received messages to the application via rx
        let (tx, rx) = channel::<IncomingMessage>();

        let l = Layers {
            encryption_layer: Arc::new(e),
            delivery_layer: Arc::new(Box::new(d)),
            console: console
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
        let console = self.console.clone();

        thread::spawn(move || { loop { match rx.recv() {
            Ok(msg) => match Layers::handle_message(msg, enc.clone(), console.clone()) {
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
    fn handle_message(m: IncomingMessage, enc: Arc<Box<dyn Encryption>>, _console: Console) -> Option<IncomingMessage> {

        // TODO error handling
        #[cfg(feature="debugout")]
            _console.status(String::from("[Layers::handle_message()] decrypting message"));

        match m {
            IncomingMessage::New(msg) => {
                #[cfg(feature="debugout")]
                    _console.send(format!("[Layers::handle_message()] new message {}", msg.buf.len())).unwrap();

                match enc.decrypt(&msg.buf) {
                    Ok(buf) => Some(IncomingMessage::New(msg.set_payload(buf))),
                    Err(_m) => {
                        #[cfg(feature="debugout")]
                            _console.status(format!("[Layers::handle_message()] decrypt returned with error. {}", _m));
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
