use time::Tm;

static MAX_BUF_LEN: usize = 500;

pub struct Model {
    /// List of all messages for the main window.
    pub buf: Vec<Item>,
    /// Content of the input field.
    pub input: Vec<u8>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            buf: vec![],
            input: vec![],
        }
    }

    /// Adds a new character (given as byte stream) to the input field.
    pub fn update_input(&mut self, buf: Vec<u8>) {
        for b in String::from_utf8(buf)
            .unwrap().chars().filter(|c| !c.is_control()).collect::<String>().as_bytes() {
            self.input.push(*b);
        }
    }

    /// Deletes one character from the input field.
    pub fn apply_backspace(&mut self) {
        loop {
            self.input.pop();
            if String::from_utf8(self.input.clone()).is_ok() {
                break;
            }
        }
    }

    pub fn apply_enter(&mut self) -> String {
        let s = String::from_utf8(self.input.clone()).expect("Invalid utf8 string.");
        self.input.clear();
        s
    }

    // Is called when we receive an ack for a file upload.
    /// `id` - id of the item in the buffer
    /// `nbytes` - number of bytes of the corresponding package that was transmitted
    pub fn ack(&mut self, id: u64) {
        for item in self.buf.iter_mut().rev() {
            let exists = item.id.iter().find(|i| **i == id).is_some();
            if exists {
                item.acks_received += 1;
                break;
            }
        }
    }

    pub fn ack_progress(&mut self, id: u64, done: usize, total: usize) {
        for item in self.buf.iter_mut().rev() {
            let exists = item.id.iter().find(|i| **i == id).is_some();
            if exists {
                item.pending_acks = done;
                item.total_acks = total;
                break;
            }
        }
    }

    pub fn add_message(&mut self, i: Item) {
        self.buf.push(i);
        // TODO not very efficient
        while self.buf.len() > MAX_BUF_LEN {
            self.buf.remove(0);
        }
    }
}

#[derive(Clone)]
pub enum Source {
    Ip(String),
    You,
    System,
    Raw,
}

#[derive(Clone)]
pub struct Item {
    pub msg: String,
    pub typ: ItemType,
    pub id: Vec<u64>,  // In group chat scenarios one item can have several IDs.
    pub acks_received: usize,
    pub tim: Tm,
    pub total_acks: usize,
    pub pending_acks: usize,
    from: Source,
}

impl Item {
    /// Creates a new item without a symbol and without an id.
    pub fn new(msg: String, typ: ItemType, from: Source) -> Item {
        Item {
            msg,
            typ,
            id: vec![],
            acks_received: 0,
            tim: time::now(),
            from,
            total_acks: 0,
            pending_acks: 0
        }
    }

    pub fn add_size(mut self, n: usize) -> Item {
        self.total_acks = n;
        self.pending_acks = n;
        self
    }

    pub fn raw(mut self) -> Item {
        self.from = Source::Raw;
        self
    }

    /// Sets the message of an item.
    pub fn message(mut self, msg: String) -> Item {
        self.msg = msg;
        self
    }

    /// Sets the id of the item.
    pub fn add_id(mut self, id: u64) -> Item {
        self.id.push(id);
        self
    }

    pub fn source(&self) -> Source {
        self.from.clone()
    }
}

// The type is used to determine the color.
#[derive(Clone)]
pub enum ItemType {
    Introduction,
    Received,
    Error,
    Info,
    NewFile,
    MyMessage,
    UploadMessage,
}
