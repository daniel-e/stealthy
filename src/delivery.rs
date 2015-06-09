extern crate rand;

use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

use super::{Message, IncomingMessage, Errors};
use binding::Network;    // Implemenation for network layer


struct SmallMessage {
    buf: Vec<u8>,
    seq: u32,
    id : u64,
    n  : u32,
}

struct SmallMessages {
    messages: Vec<SmallMessage>,
    ip: String,
    id: u64
}

pub struct Delivery {
    pending      : Arc<Mutex<Vec<SmallMessages>>>,
    tx           : Sender<IncomingMessage>,
    network_layer: Box<Network>
}

const MAX_MESSAGE_PART_SIZE: usize = 128;

impl Delivery {

    /// Via rx1 this layer receives incoming messages from the
    /// network layer.
    pub fn new(n: Box<Network>, tx: Sender<IncomingMessage>, rx: Receiver<IncomingMessage>) -> Delivery {

        let mut d = Delivery {
            pending: Arc::new(Mutex::new(vec![])),
            tx: tx,
            network_layer: n
        };

        d.init_rx(rx);
        d
    }

    fn init_rx(&self, rx: Receiver<IncomingMessage>) {

        let tx = self.tx.clone();
        let queue = self.pending.clone();

		thread::spawn(move || { loop { 
            let r = rx.recv();
            match r {
                // TODO handle splitted messages
                Ok(msg) => { tx.send(msg); }
                _ => { } // TODO
            }
        }});
    }

    pub fn send_msg(&self, msg: Message) -> Result<u64, Errors> {

        let queue = self.pending.clone();
        let mut pending = queue.lock().unwrap();

        let small_messages = self.split_message(&msg);
        pending.push(small_messages);

        // TODO
        self.network_layer.send_msg(msg)
    }

    fn split_message(&self, msg: &Message) -> SmallMessages {

        let id = rand::random::<u64>();
        let mut parts: Vec<SmallMessage> = Vec::new();
        let mut i: u32 = 1;

        let chunks = msg.buf.chunks(MAX_MESSAGE_PART_SIZE);
        let n = chunks.len();

        for win in chunks {
            parts.push(SmallMessage {
                buf: win.to_vec(),
                seq: i,
                id: id,
                n: n as u32,
            });
            i += 1;
        }

        SmallMessages {
            messages: parts,
            id: id,
            ip: msg.get_ip()
        }
    }

    /// Serializes a chunk into a vector which is ready to be transmitted
    /// via an icmp echo request.
    fn serialize(m: &SmallMessage) -> Vec<u8> {

        let mut v: Vec<u8> = Vec::new();
        v.push(1);                          // version u8
        push_val(&mut v, m.id, 8);          // id u64
        push_val(&mut v, m.n as u64, 4);    // number of messages u32
        push_val(&mut v, m.seq as u64, 4);  // seq u32
        push_slice(&mut v, &m.buf);         // message: variable len
        v
    }

    /// Deserialized a received icmp echo request into a chunk.
    fn deserialize(data: &Vec<u8>) -> Option<SmallMessage> {

        if data.len() < (1 + 8 + 4 + 4) {
            return None;
        }

        let mut v = data.clone();
        let version = pop_val(&mut v, 1).unwrap();

        if version != 1 {
            return None;
        }

        let id: u64 = pop_val(&mut v, 8).unwrap();         // id
        let n: u32 = pop_val(&mut v, 4).unwrap() as u32;   // number of messages
        let seq: u32 = pop_val(&mut v, 4).unwrap() as u32; // seq
        
        Some(SmallMessage {
            buf: v.clone(),
            seq: seq,
            id : id,
            n  : n
        })
    }
}

fn push_slice(v: &mut Vec<u8>, arr: &[u8]) {
    for i in arr { 
        v.push(*i) 
    }
}

fn push_val(dst: &mut Vec<u8>, val: u64, n: usize) {
    let mut v = val;
    let mask = 0xff as u64;
    for _ in 0..n {
        let x: u8 = (v & mask) as u8;
        dst.push(x);
        v = v >> 8;
    }
}

fn pop_val(src: &mut Vec<u8>, n: usize) -> Option<u64> {
    let mut r: u64 = 0;
    if src.len() < n {
        return None;
    }
    for i in 0..n {
        r = r << 8;
        r = r + (src[n - 1 - i] as u64);
        src.remove(n - 1 - i); // TODO performance
    }
    Some(r)
}


// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::{Delivery, MAX_MESSAGE_PART_SIZE, SmallMessage};
    use std::iter::FromIterator;

    use ::Message;

    #[test]
    fn test_new() {
        
        let d = Delivery::new();
        assert_eq!(d.pending.len(), 0);
    }

    #[test]
    fn test_split_small_message() {
        
        let data = "hallo".to_string().into_bytes();
        let msg  = Message::new("1.2.3.4".to_string(), data.clone());
        let s    = Delivery::new();
        let r    = s.split_message(&msg);

        // check that the IP is the same as in the message.
        assert_eq!(r.ip, "1.2.3.4".to_string());
        // Check that a random id has been generated.
        assert!(r.id != 0);
        // Check that there is one message.
        assert!(r.messages.len() == 1);
        // Check that the sequence number of the first message is 1.
        assert!(r.messages[0].seq == 1);
        // Check that the first message is equal to the original message.
        assert_eq!(r.messages[0].buf, data);
        
        assert_eq!(r.messages[0].id, r.id);
        assert_eq!(r.messages[0].n, 1);
    }

    #[test]
    fn test_split_small_message_a() {

        let v = (0..MAX_MESSAGE_PART_SIZE).map(|x| x as u8).collect::<Vec<_>>();
        let m = Message::new("1.2.3.4".to_string(), v.clone());
        let d = Delivery::new();
        let r = d.split_message(&m);

        assert_eq!(r.messages.len(), 1);
        assert_eq!(r.messages[0].buf, v);
        assert_eq!(r.messages[0].id, r.id);
        assert_eq!(r.messages[0].n, 1);
    }

    #[test]
    fn test_split_big_message() {

        // Create a message that should be divided into two pieces.
        let v = (0..MAX_MESSAGE_PART_SIZE + 1).map(|x| x as u8).collect::<Vec<_>>();
        let m = Message::new("1.2.3.4".to_string(), v.clone());
        let d = Delivery::new();
        let r = d.split_message(&m);

        assert_eq!(r.ip, "1.2.3.4".to_string());
        assert!(r.id != 0);
        assert!(r.messages.len() == 2);
        assert!(r.messages[0].seq == 1);
        assert!(r.messages[0].id == r.id);
        assert!(r.messages[0].n == 2);
        assert!(r.messages[1].seq == 2);
        assert!(r.messages[1].id == r.id);
        assert!(r.messages[1].n == 2);
        assert!(r.messages[0].buf.len() == super::MAX_MESSAGE_PART_SIZE);
        assert!(r.messages[1].buf.len() == 1);

        let (v1, v2) = v.split_at(super::MAX_MESSAGE_PART_SIZE);
        assert_eq!(r.messages[0].buf, v1);
        assert_eq!(r.messages[1].buf, v2);
    }

    #[test]
    fn test_de_and_serialize() {

        let mp = SmallMessage {
            buf: vec![1, 2, 3, 8, 9],
            seq: 211 * 256 + 189,
            n  : (99 * 256 + 134) * 256 + 177,
            id : (12 * 256 + 19) * 256 + 18,
        };

        let v = Delivery::serialize(&mp);
        assert_eq!(v, vec![
                1,                         // version
                18, 19, 12, 0, 0, 0, 0, 0, // Id
                177, 134, 99, 0,           // total
                189, 211, 0, 0,            // seq
                1, 2, 3, 8, 9              // msg
            ]);

        let m = Delivery::deserialize(&v);

        assert!(m.is_some());
        let p = m.unwrap();
        assert_eq!(p.id, (12 * 256 + 19) * 256 + 18);
        assert_eq!(p.seq, 211 * 256 + 189);
        assert_eq!(p.n, (99 * 256 + 134) * 256 + 177);

        // Check that length check does work.
        let mut x: Vec<u8> = vec![1, 2];
        assert!(!Delivery::deserialize(&x).is_some());

        // Check that version check does work.
        x = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(!Delivery::deserialize(&x).is_some());

        // Check that version check does work.
        x = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        x = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(Delivery::deserialize(&x).is_some());

        // Check that length check does work.
        x = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(Delivery::deserialize(&x).is_some());
    }

    // ========================================================================

    use super::{push_slice, push_val, pop_val};

    #[test]
    fn test_push_slice() {
        let mut v: Vec<u8> = Vec::new();
        push_slice(&mut v, &[1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_push_val() {
        let mut v: Vec<u8> = Vec::new();

        push_val(&mut v, 123, 2);
        assert_eq!(v, vec![123, 0]);

        v.clear();
        push_val(&mut v, 23 * 256 + 78, 2);
        assert_eq!(v, vec![78, 23]);
    }

    #[test]
    fn test_pop_val() {
        let mut v: Vec<u8> = vec![1, 2, 3];
        
        let mut i = pop_val(&mut v, 4);
        assert!(!i.is_some());

        v.clear();
        push_val(&mut v, 17 * 256 + 19, 2);
        push_val(&mut v, 34, 1);
        i = pop_val(&mut v, 2);
        assert_eq!(i.unwrap(), 17 * 256 + 19);
        assert_eq!(v.len(), 1);

        i = pop_val(&mut v, 1);
        assert_eq!(i.unwrap(), 34);
        assert_eq!(v.len(), 0);
    }
}

