use std::io::stdin;
use std::sync::mpsc::channel;
use std::thread;
use std::io::Read;
use std::iter::repeat;
use std::time::Duration;
use std::sync::mpsc::Receiver;

pub enum UserInput {
    Character(Vec<u8>),
    Escape,
    CtrlD,
    Enter,
    Backspace,
    ArrowUp,
    ArrowDown,
    End,
    PageDown,
    PageUp,
    CtrlR
}

/// Use to receive user input.
pub struct TermIn {
    rx: Receiver<u8>,
}

impl TermIn {

    pub fn new() -> TermIn {

        // The sender tx is used by the thread below to send bytes from stdin
        // to the receiver rx. The method next_char listens on the receiver.
        let (tx, rx) = channel();

        thread::spawn(move || {
            for b in stdin().bytes() {
                tx.send(b.unwrap()).expect("Error.");
            }
        });

        TermIn { rx }
    }

    fn next_char(&self) -> Vec<u8> {

        self.rx.recv()
            .into_iter()
            .chain(repeat(0)
                .map(|_| self.rx.recv_timeout(Duration::from_millis(2)))
                .take_while(Result::is_ok)
                .map(Result::unwrap)
            )
            .collect()
    }

    fn map_input(buf: Vec<u8>) -> Option<UserInput> {

        //println!("{:?}", buf);
        //thread::sleep(Duration::from_secs(3));

        if buf == vec![27] {                 // Escape
            Some(UserInput::Escape)
        } else if buf == vec![4] {           // Ctrl + D
            Some(UserInput::CtrlD)
        } else if buf == vec![13] {          // Enter
            Some(UserInput::Enter)
        } else if buf == vec![127] {         // Backspace
            Some(UserInput::Backspace)
        } else if buf == vec![27, 91, 65] {  // Arrow up
            Some(UserInput::ArrowUp)
        } else if buf == vec![27, 91, 66] {  // Arrow down
            Some(UserInput::ArrowDown)
        } else if buf == vec![27, 91, 70] {  // End
            Some(UserInput::End)
        } else if buf == vec![18] {          // Ctrl + R
            Some(UserInput::CtrlR)
        } else if buf.len() < 3 {            // Some character
            Some(UserInput::Character(buf))
        } else if buf == vec![27, 91, 53, 126] { // Page up
            Some(UserInput::PageUp)
        } else if buf == vec![27, 91, 54, 126] { // Page down
            Some(UserInput::PageDown)
        } else {
            None
        }
    }

    pub fn read_char(&mut self) -> UserInput {

        loop {
            let i = Self::map_input(self.next_char());
            if i.is_some() {
                return i.unwrap();
            }
        }
    }
}
