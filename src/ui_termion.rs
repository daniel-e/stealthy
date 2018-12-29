use std::sync::Arc;
use std::sync::Mutex;
use std::io::{self, BufRead};

use termion;
use crate::console::Color;
use std::io::stdout;
use termion::raw::IntoRawMode;
use std::io::Write;
use termion::raw::RawTerminal;
use std::io::StdoutLock;
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

pub struct Screen {

}

impl Screen {
    pub fn new() -> Screen {
        Screen { }
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
    scr: Arc<Mutex<Screen>>,
    model: Arc<Mutex<Model>>,
    rx: Receiver<u8>,
}

impl TermOut {

    pub fn new(model: Arc<Mutex<Model>>) -> TermOut {

        let mut stdout = stdout().into_raw_mode().expect("No raw mode.");

        write!(stdout, "{}", termion::clear::All);
        stdout.flush().unwrap();

        TermOut {
            stdout: stdout,
            model: model
        }
    }

    pub fn close(&self) {

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
            );
        }
        for y in 2..maxy {
            write!(self.stdout, "{}│{}│",
                   termion::cursor::Goto(1, y),
                   termion::cursor::Goto(maxx, y)
            );
        }
        write!(self.stdout,
               "{}┌{}┐{}└{}┘{}├{}┤",
               termion::cursor::Goto(1, 1),
               termion::cursor::Goto(maxx, 1),
               termion::cursor::Goto(1, maxy),
               termion::cursor::Goto(maxx, maxy),
               termion::cursor::Goto(1, maxy - 2),
               termion::cursor::Goto(maxx, maxy - 2)
        );
    }

    fn redraw(&mut self) {
        self.draw_window();

        {
            let buf = &self.model.lock().unwrap().buf;
            let mut y = 2;
            for e in buf {
                write!(self.stdout, "{}", termion::cursor::Goto(2, y));
                y += 1;
                write!(self.stdout, "{}", e);
            }
        }

        // draw input field
        {
            let (maxx, maxy) = TermOut::size();
            write!(self.stdout, "{}", termion::color::Bg(termion::color::Blue));
            for x in 2..maxx {
                write!(self.stdout, "{} ",
                       termion::cursor::Goto(x, maxy - 1),

                );
            }
            let input = self.model.lock().unwrap().input.clone();
            let s = String::from_utf8(input).unwrap();

            write!(self.stdout, "{}{}", termion::cursor::Goto(2, maxy - 1), s);
            write!(self.stdout, "{}", termion::color::Bg(termion::color::Reset));
        }

        self.stdout.flush().unwrap();
    }
}

impl TermIn {

    pub fn new(scr: Arc<Mutex<Screen>>, model: Arc<Mutex<Model>>) -> TermIn {

        let (tx, rx) = channel();

        thread::spawn(move || {
            let stdin = stdin();
            let stdin = stdin.lock();
            let mut bytes = stdin.bytes();
            loop {
                let b = bytes.next().unwrap().unwrap();
                tx.send(b);
            }
        });

        TermIn {
            scr: scr,
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
                        Ok(b) => {
                            buf.push(b);
                        },
                        _ => {
                            break;
                        }
                    }
                }
            },
            _ => { return None; }
        };

        if buf.len() == 1 {
            if buf[0] == 27 { // Escape
                return None;
            }
        }

        for b in buf {
            self.model.lock().unwrap().input.push(b);
        }
        Some(UserInput::Refresh)
    }
}


