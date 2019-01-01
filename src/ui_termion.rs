use std::sync::Arc;
use std::sync::Mutex;

use termion;
use std::io::stdout;
use termion::raw::IntoRawMode;
use std::io::Write;
use termion::raw::RawTerminal;
use std::io::Stdout;
use std::io::stdin;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::cmp::min;
use termion::color::Fg;

static TRANSMITTING: char = '◷';
static ACK: char = '✔';

#[derive(Clone)]
pub enum ItemType {
    Introduction,
    Received,
    Error,
    Info,
    NewFile,
    MyMessage,
}

#[derive(Clone)]
pub enum Symbol {
    Transmitting,
    Ack,
}

#[derive(Clone)]
pub struct Item {
    pub msg: String,
    typ: ItemType,
    symbol: Option<Symbol>,
    id: Option<u64>,
}

impl Item {
    pub fn new(msg: String, typ: ItemType) -> Item {
        Item {
            msg,
            typ,
            symbol: None,
            id: None
        }
    }

    pub fn symbol(mut self, s: Symbol) -> Item {
        self.symbol = Some(s);
        self
    }

    pub fn message(mut self, msg: String) -> Item {
        self.msg = msg;
        self
    }

    pub fn id(mut self, id: u64) -> Item {
        self.id = Some(id);
        self
    }
}

pub struct Model {
    buf: Vec<Item>,
    input: Vec<u8>,
    scroll_offset: usize,
}

impl Model {
    pub fn new() -> Model {
        Model {
            buf: vec![],
            input: vec![],
            scroll_offset: 0,
        }
    }

    //pub fn messages(&self) -> Vec<String> {
    //    self.buf.iter().map(|x| x.msg.clone()).collect()
    //}
}

pub enum ControlType {
    ArrowUp,
    ArrowDown,
}

pub enum UserInput {
    Line(String),
    Control(ControlType),
    Refresh,
}


pub struct TermOut {
    stdout: RawTerminal<Stdout>,
    model: Arc<Mutex<Model>>,
}

pub struct TermIn {
    model: Arc<Mutex<Model>>,
    rx: Receiver<u8>,
}

impl TermOut {

    pub fn new(model: Arc<Mutex<Model>>) -> TermOut {

        TermOut {
            stdout: stdout().into_raw_mode().expect("No raw mode possible."),
            model: model
        }.init()
    }

    pub fn close(&mut self) {
        write!(self.stdout, "{}{}{}",
               termion::clear::All,
               termion::cursor::Goto(1, 1),
               termion::cursor::Show
        ).expect("Write error.");
        self.flush();
    }

    pub fn ack(&mut self, id: u64) {
        {
            let mut model = self.model.lock().unwrap();
            for i in model.buf.iter_mut().rev() {
                match i.id {
                    Some(mid) => {
                        if mid == id {
                            i.symbol = Some(Symbol::Ack);
                            break;
                        }
                    },
                    _ => {}
                }
            }
        }
        self.redraw();
    }

    pub fn println(&mut self, i: Item) {

        {
            let mut model = self.model.lock().unwrap();
            if model.scroll_offset > 0 {
                model.scroll_offset += TermOut::split_line(&i).len();
            }
            model.buf.push(i);
        }
        self.redraw();
    }

    pub fn scroll_up_1(model: &mut Model) {
        let window_height = TermOut::window_height();
        let buffer_lines = TermOut::lines(&model.buf).len();

        if buffer_lines > window_height {
            let max_off = buffer_lines - window_height;
            model.scroll_offset = min(max_off, model.scroll_offset + 1);
        }
    }

    pub fn scroll_down_1(model: &mut Model) {
        if model.scroll_offset > 0 {
            model.scroll_offset -= 1;
        }
    }

    pub fn scroll_up(&mut self) {
        TermOut::scroll_up_1(&mut self.model.lock().unwrap());
        self.refresh();
    }

    pub fn scroll_down(&mut self) {
        TermOut::scroll_down_1(&mut self.model.lock().unwrap());
        self.refresh();
    }

    pub fn refresh(&mut self) {
        self.redraw();
    }

    // ===========================================================================================

    fn init(mut self) -> TermOut {
        write!(self.stdout, "{}{}",
               termion::clear::All,   // clear screen
               termion::cursor::Hide  // hide cursor
        ).expect("Error.");
        self.flush();
        self
    }

    fn flush(&mut self) {
        self.stdout.flush().expect("Flush error.");
    }

    // ===========================================================================================

    fn size() -> (u16, u16) {
        termion::terminal_size().unwrap()
    }

    fn draw_window(&mut self) {
        let (maxx, maxy) = TermOut::size();

        for x in 2..maxx {
            write!(self.stdout, "{}─{}─{}─",
                   termion::cursor::Goto(x, 1),
                   termion::cursor::Goto(x, maxy),
                   termion::cursor::Goto(x, maxy - 2),
            ).expect("Error.");
        }
        for y in 2..maxy {
            write!(self.stdout, "{}│{}│",
                   termion::cursor::Goto(1, y),
                   termion::cursor::Goto(maxx, y)
            ).expect("Error.");
        }
        write!(self.stdout,
               "{}┌{}┐{}└{}┘{}├{}┤",
               termion::cursor::Goto(1, 1),
               termion::cursor::Goto(maxx, 1),
               termion::cursor::Goto(1, maxy),
               termion::cursor::Goto(maxx, maxy),
               termion::cursor::Goto(1, maxy - 2),
               termion::cursor::Goto(maxx, maxy - 2)
        ).expect("Error.");
    }

    fn window_height() -> usize {
        TermOut::size().1 as usize - 4
    }

    fn window_width() -> usize {
        TermOut::size().0 as usize - 2
    }

    fn split_line(s: &Item) -> Vec<Item> {
        // TODO use https://github.com/unicode-rs/unicode-width to estimate the width of UTF-8 characters
        s.msg.chars().collect::<Vec<char>>()
            .chunks(TermOut::window_width())
            .map(|x| s.clone().message(x.iter().collect()))
            .collect()
    }

    fn lines(buf: &Vec<Item>) -> Vec<Item> {
        buf.iter()
            .map(|v| TermOut::split_line(v))
            .flatten()
            .collect()
    }

    fn redraw(&mut self) {
        self.draw_window();

        let model = self.model.lock().unwrap();
        let screen_width = TermOut::window_width();
        let screen_height = TermOut::window_height();

        // Write buffer to screen.

        let buffer = TermOut::lines(&model.buf);
        let n = buffer.len();
        let mut p = 0;
        let buf = if n <= screen_height {
            buffer.clone()
        } else {
            // n - screen_height: index for scroll_offset = 0
            p = n - screen_height - model.scroll_offset;
            buffer.iter().skip(p).take(screen_height).cloned().collect()
        };

        for (y, line) in buf.iter().enumerate() {
            let mut s = line.clone();
            while s.msg.chars().count() < screen_width {
                s.msg.push(' ');
            }

            match s.typ {
                ItemType::Received => write!(self.stdout, "{}", Fg(termion::color::LightGreen)).unwrap(),
                ItemType::Info => write!(self.stdout, "{}", Fg(termion::color::Yellow)).unwrap(),
                ItemType::Introduction => write!(self.stdout, "{}", Fg(termion::color::Green)).unwrap(),
                ItemType::Error => write!(self.stdout, "{}", Fg(termion::color::Red)).unwrap(),
                ItemType::NewFile => write!(self.stdout, "{}", Fg(termion::color::LightWhite)).unwrap(),
                ItemType::MyMessage => write!(self.stdout, "{}", Fg(termion::color::Green)).unwrap(),
            };

            write!(self.stdout, "{}{}{}",
                   termion::cursor::Goto(2, y as u16 + 2),
                   s.msg,
                   termion::color::Fg(termion::color::Reset)
            ).expect("Error.");

            match s.symbol {
                Some(symbol) => {
                    match symbol {
                        Symbol::Transmitting => {
                            write!(self.stdout, "{}{}{}{}",
                                   Fg(termion::color::LightYellow),
                                   termion::cursor::Goto(16, y as u16 + 2),
                                   TRANSMITTING,
                                   termion::color::Fg(termion::color::Reset)
                            ).expect("Error.");
                        },
                        Symbol::Ack => {
                            write!(self.stdout, "{}{}{}{}",
                                   Fg(termion::color::Green),
                                   termion::cursor::Goto(16, y as u16 + 2),
                                   ACK,
                                   termion::color::Fg(termion::color::Reset)
                            ).expect("Error.");
                        }
                    }
                },
                _ => {}
            }
        }


        let (maxx, maxy) = TermOut::size();

        // Write input field to screen.
        let input_field_len = maxx - 2 - 1; // one character for cursor

        write!(self.stdout, "{}", termion::color::Bg(termion::color::Blue)).expect("Error.");
        for x in 2..maxx {
            write!(self.stdout, "{} ", termion::cursor::Goto(x, maxy - 1)).expect("Error.");
        }

        let mut s = String::from_utf8(model.input.clone()).unwrap();
        while s.chars().count() > input_field_len as usize {
            s.remove(0);

        }
        s.push('▂');
        write!(self.stdout, "{}{}{}",
               termion::cursor::Goto(2, maxy - 1),
               s,
               termion::color::Bg(termion::color::Reset)
        ).expect("Error.");

        // Scroll status.
        if model.scroll_offset > 0 {
            let s = format!("line:{}/{}", p, buffer.len());
            let x = maxx as usize - s.len();
            write!(self.stdout, "{}{}{}{}{}{}",
                   termion::cursor::Goto(x as u16, 2),
                   termion::color::Bg(termion::color::Red),
                   termion::color::Fg(termion::color::LightWhite),
                   s,
                   termion::color::Bg(termion::color::Reset),
                   termion::color::Fg(termion::color::Reset)
            ).expect("Error.");
        }

        self.stdout.flush().unwrap();
    }
}

impl TermIn {

    pub fn new(model: Arc<Mutex<Model>>) -> TermIn {

        let (tx, rx) = channel();

        thread::spawn(move || {
            let stdin = stdin();
            let stdin = stdin.lock();
            let mut bytes = stdin.bytes();
            loop {
                let b = bytes.next().unwrap().unwrap();
                tx.send(b).expect("Error.");
            }
        });

        TermIn {
            model: model,
            rx: rx,
        }
    }

    pub fn read_char(&mut self) -> Option<UserInput> {
        let mut buf = vec![];

        match self.rx.recv() {
            Ok(b) => {
                buf.push(b);
                loop {
                    match self.rx.recv_timeout(Duration::from_millis(2)) {
                        Ok(b) => { buf.push(b); },
                        _ => { break; }
                    }
                }
            },
            _ => { return None; }
        };

        let mut model = self.model.lock().unwrap();

        if buf == vec![27] {         // Escape
            return None;
        } else if buf == vec![4] {   // Ctrl + D
            return None;
        } else if buf == vec![13] {  // Enter
            let s = String::from_utf8(model.input.clone()).unwrap();
            model.input.clear();
            return Some(UserInput::Line(s));
        } else if buf == vec![127] { // backspace
            loop {
                model.input.pop();
                let s = String::from_utf8(model.input.clone());
                if s.is_ok() {
                    break;
                }
            }
            return Some(UserInput::Refresh);
        } else if buf == vec![27, 91, 65] {  // Arrow up
            Some(UserInput::Control(ControlType::ArrowUp))
        } else if buf == vec![27, 91, 66] {  // Arrow down
            Some(UserInput::Control(ControlType::ArrowDown))
        } else if buf == vec![27, 91, 70] {  // End
            model.scroll_offset = 0;
            Some(UserInput::Refresh)
        } else if buf.len() < 3 {
            for b in String::from_utf8(buf)
                .unwrap().chars().filter(|c| !c.is_control()).collect::<String>().as_bytes() {
                model.input.push(*b);
            }
            Some(UserInput::Refresh)
        } else if buf == vec![27, 91, 53, 126] { // Page up
            for _ in 0..TermOut::window_height() {
                TermOut::scroll_up_1(&mut model);
            }
            Some(UserInput::Refresh)
        } else if buf == vec![27, 91, 54, 126] { // Page down
            for _ in 0..TermOut::window_height() {
                TermOut::scroll_down_1(&mut model);
            }
            Some(UserInput::Refresh)
        } else {
            //println!("{}, {:?}", buf.len(), buf);
            //self.rx.recv();
            Some(UserInput::Refresh)
        }
    }
}


