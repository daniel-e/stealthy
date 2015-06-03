extern crate rand;

use super::Message;
use super::tools;

struct MessagePart {
    buf: Vec<u8>,
    seq: u32,
    n  : u32,
    id : u64
}

struct Delivery {
    messages: Vec<MessagePart>,
    id: u64,
    ip: String
}

const MAX_MESSAGE_PART_SIZE: usize = 128;

impl Delivery {

    pub fn new(msg: &Message) -> Delivery {
        
        let     id = rand::random::<u64>();
        let mut parts: Vec<MessagePart> = Vec::new();
        let mut i: u32 = 1;

        let chunks = msg.buf.chunks(MAX_MESSAGE_PART_SIZE);
        let n = chunks.len();

        for win in chunks {
            parts.push(MessagePart {
                buf: win.to_vec(),
                seq: i,
                n  : n as u32,
                id : id
            });
            i += 1;
        }

        Delivery {
            messages: parts,
            id: id,
            ip: msg.ip.clone()
        }
    }

    fn serialize(m: &MessagePart) -> Vec<u8> {

        let mut v: Vec<u8> = Vec::new();
        v.push(1);                                 // version u8
        tools::push_val(&mut v, m.id, 8);          // id u64
        tools::push_val(&mut v, m.n as u64, 4);    // number of messages u32
        tools::push_val(&mut v, m.seq as u64, 4);  // seq u32
        tools::push_slice(&mut v, &m.buf);         // message: variable len
        v
    }

    fn deserialize(data: &Vec<u8>) -> Option<MessagePart> {

        if data.len() < (1 + 8 + 4 + 4) {
            return None;
        }

        let mut v = data.clone();
        let version = tools::pop_val(&mut v, 1).unwrap();

        if version != 1 {
            return None;
        }

        let id: u64 = tools::pop_val(&mut v, 8).unwrap();         // id
        let n: u32 = tools::pop_val(&mut v, 4).unwrap() as u32;   // number of messages
        let seq: u32 = tools::pop_val(&mut v, 4).unwrap() as u32; // seq
        
        Some(MessagePart {
            buf: v.clone(),
            seq: seq,
            n  : n,
            id : id
        })
    }
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::{Delivery, MAX_MESSAGE_PART_SIZE, MessagePart};
    use std::iter::FromIterator;

    use ::network::Message;

    #[test]
    fn test_new_id() {
        let m = Message::new("1.2.3.4".to_string(), vec![]);
        let m1 = Delivery::new(&m);
        let m2 = Delivery::new(&m);
        // it should be very unlikely that an ID is equal to zero
        assert!(m1.id != 0);
        assert!(m2.id != 0);
        // check that messages have different IDs
        assert!(m1.id != m2.id);
    }

    #[test]
    fn test_new_small_message() {
        
        let data = "hallo".to_string().into_bytes();
        let msg = Message::new("1.2.3.4".to_string(), data.clone());
        let s = Delivery::new(&msg);

        // check that the IP is the same as in the message.
        assert_eq!(s.ip, "1.2.3.4".to_string());
        // Check that a random id has been generated.
        assert!(s.id != 0);
        // Check that there is one message.
        assert!(s.messages.len() == 1);
        // Check that the sequence number of the first message is 1.
        assert!(s.messages[0].seq == 1);
        // Check that the first message is equal to the original message.
        assert_eq!(s.messages[0].buf, data);
        
        assert_eq!(s.messages[0].id, s.id);
        assert_eq!(s.messages[0].n, 1);
    }

    #[test]
    fn test_new_one_message() {

        let v = (0..MAX_MESSAGE_PART_SIZE).map(|x| x as u8).collect::<Vec<_>>();
        let m = Message::new("1.2.3.4".to_string(), v.clone());
        let d = Delivery::new(&m);

        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].buf, v);
        assert_eq!(d.messages[0].id, d.id);
        assert_eq!(d.messages[0].n, 1);
    }

    #[test]
    fn test_new_big_message() {

        // Create a message that should be divided into two
        // pieces.
        let piece = "0123456789".to_string().into_bytes();
        let mut data: Vec<u8> = Vec::new();
        for _ in 0..20 {
            for i in piece.clone() { data.push(i); }
        }
        let msg = Message::new("1.2.3.4".to_string(), data.clone());
        let s = Delivery::new(&msg);

        assert_eq!(s.ip, "1.2.3.4".to_string());
        assert!(s.id != 0);
        assert!(s.messages.len() == 2);
        assert!(s.messages[0].seq == 1);
        assert!(s.messages[0].id == s.id);
        assert!(s.messages[0].n == 2);
        assert!(s.messages[1].seq == 2);
        assert!(s.messages[1].id == s.id);
        assert!(s.messages[1].n == 2);
        assert!(s.messages[0].buf.len() == super::MAX_MESSAGE_PART_SIZE);
        assert!(s.messages[1].buf.len() == data.len() - super::MAX_MESSAGE_PART_SIZE);

        let (v1, v2) = data.split_at(super::MAX_MESSAGE_PART_SIZE);
        assert_eq!(s.messages[0].buf, v1);
        assert_eq!(s.messages[1].buf, v2);
    }

    #[test]
    fn test_de_and_serialize() {

        let mp = MessagePart {
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
}

