extern crate rand;

use ::network::Message;
use std::iter::FromIterator;

struct MessagePart {
    buf: Vec<u8>,
    seq: u32
}

struct MessageSplitter {
    messages: Vec<MessagePart>,
    id: u64,
    ip: String
}

const MAX_MESSAGE_PART_SIZE: usize = 128;

impl MessageSplitter {

    pub fn new(msg: Message) -> MessageSplitter {
        
        let mut parts: Vec<MessagePart> = Vec::new();
        let mut i: u32 = 0;

        for win in msg.buf.chunks(MAX_MESSAGE_PART_SIZE) {
            parts.push(MessagePart {
                buf: win.to_vec(),
                seq: i
            });
            i += 1;
        }

        MessageSplitter {
            messages: parts,
            id: rand::random::<u64>(),
            ip: msg.ip
        }
    }
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::MessageSplitter;
    use ::network::Message;

    #[test]
    fn test_new_id() {
        let m1 = MessageSplitter::new(Message::new("1.2.3.4".to_string(), vec![]));
        let m2 = MessageSplitter::new(Message::new("1.2.3.4".to_string(), vec![]));
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
        let s = MessageSplitter::new(msg);

        assert_eq!(s.ip, "1.2.3.4".to_string());
        assert!(s.id != 0);
        assert!(s.messages.len() == 1);
        assert!(s.messages[0].seq == 0);
        assert_eq!(s.messages[0].buf, data);
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
        let s = MessageSplitter::new(msg);

        assert_eq!(s.ip, "1.2.3.4".to_string());
        assert!(s.id != 0);
        assert!(s.messages.len() == 2);
        assert!(s.messages[0].seq == 0);
        assert!(s.messages[1].seq == 1);
        assert!(s.messages[0].buf.len() == super::MAX_MESSAGE_PART_SIZE);
        assert!(s.messages[1].buf.len() == data.len() - super::MAX_MESSAGE_PART_SIZE);

        let (v1, v2) = data.split_at(super::MAX_MESSAGE_PART_SIZE);
        assert_eq!(s.messages[0].buf, v1);
        assert_eq!(s.messages[1].buf, v2);
    }
}

