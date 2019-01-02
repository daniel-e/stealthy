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

    pub fn ack(&mut self, id: u64) {
        for i in self.buf.iter_mut().rev() {
            if i.id.is_some() && i.id.unwrap() == id {
                i.symbol = Some(Symbol::Ack);
                break;
            }
        }
    }

    pub fn add_message(&mut self, i: Item) {
        self.buf.push(i);
    }
}

#[derive(Clone)]
pub struct Item {
    pub msg: String,
    pub typ: ItemType,
    pub symbol: Option<Symbol>,
    pub id: Option<Vec<u64>>, // In group chat settings one item can have several IDs.
    pub acks_received: usize,
}

impl Item {
    /// Creates a new item without a symbol and without an id.
    pub fn new(msg: String, typ: ItemType) -> Item {
        Item {
            msg,
            typ,
            symbol: None,
            id: None,
            acks_received: 0
        }
    }

    /// Sets the symbol of an item.
    pub fn symbol(mut self, s: Symbol) -> Item {
        self.symbol = Some(s);
        self
    }

    /// Sets the message of an item.
    pub fn message(mut self, msg: String) -> Item {
        self.msg = msg;
        self
    }

    /// Sets the id of the item.
    pub fn id(mut self, id: u64) -> Item {
        self.id = Some(id);
        self
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
}

// This is the status symbol to display for a message.
#[derive(Clone)]
pub enum Symbol {
    Transmitting,
    Ack,
}
