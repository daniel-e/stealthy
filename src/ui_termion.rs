use std::sync::Arc;
use std::sync::Mutex;

use termion;
use crate::console::Color;
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


pub struct Model {
    buf: Vec<String>,
    input: Vec<u8>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            buf: vec![],
            input: vec![]
        }
    }
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

        let mut stdout = stdout().into_raw_mode().expect("No raw mode.");

        write!(stdout, "{}", termion::clear::All).expect("Error.");
        write!(stdout, "{}", termion::cursor::Hide).expect("Error.");
        stdout.flush().unwrap();

        TermOut {
            stdout: stdout,
            model: model
        }
    }

    pub fn close(&mut self) {
        write!(self.stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).expect("Error.");
        write!(self.stdout, "{}", termion::cursor::Show).expect("Error.");
        self.stdout.flush().unwrap();
    }

    pub fn println(&mut self, s: String, color: Color) {

        self.model.lock().unwrap().buf.push(s);
        self.redraw();
    }

    pub fn scroll_up(&mut self) {
    }

    pub fn scroll_down(&mut self) {
    }

    pub fn refresh(&mut self) {
        self.redraw();
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

    fn redraw(&mut self) {
        self.draw_window();

        let model = self.model.lock().unwrap();

        // Write buffer to screen.
        let buf = &model.buf;
        let mut y = 2;
        for e in buf {
            write!(self.stdout, "{}", termion::cursor::Goto(2, y)).expect("Error.");
            y += 1;
            write!(self.stdout, "{}", e).expect("Error.");
        }

        // Write input field to screen.
        let (maxx, maxy) = TermOut::size();
        let input_field_len = (maxx - 2 - 1) as usize; // one character for cursor

        write!(self.stdout, "{}", termion::color::Bg(termion::color::Blue)).expect("Error.");
        for x in 2..maxx {
            write!(self.stdout, "{} ", termion::cursor::Goto(x, maxy - 1)).expect("Error.");
        }

        let mut s = String::from_utf8(model.input.clone()).unwrap();
        while s.chars().count() > input_field_len {
            s.remove(0);

        }
        s.push('▂');
        write!(self.stdout, "{}{}{}",
               termion::cursor::Goto(2, maxy - 1),
               s,
               termion::color::Bg(termion::color::Reset)
        ).expect("Error.");

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

        //println!("{}, {:?}", buf.len(), buf);
        //self.rx.recv();

        if buf.len() == 1 {

            if buf[0] == 27 { // Escape
                return None;
            } else if buf[0] == 4 { // Ctrl + D
                return None;
            } else if buf[0] == 13 { // Enter
                let s = String::from_utf8(model.input.clone()).unwrap();
                model.input.clear();
                return Some(UserInput::Line(s));
            } else if buf[0] == 127 { // backspace
                loop {
                    model.input.pop();
                    let s = String::from_utf8(model.input.clone());
                    if s.is_ok() {
                        break;
                    }
                }
                return Some(UserInput::Refresh);
            }
        }

        for b in buf {
            model.input.push(b);
        }

        Some(UserInput::Refresh)
    }
}


