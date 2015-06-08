extern crate term;
extern crate ncurses;

use std::io;
use term::color;
use self::ncurses::*;

use humaninterface::InputOutput;
use callbacks::Callbacks;

pub struct Ncurses {
    maxx: i32,
    maxy: i32,
}

impl Ncurses {

    pub fn new() -> Ncurses {

        initscr();
        clear();
        noecho();
        refresh();

        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);

        for x in 0..max_x {
            mv(max_y - 2, x);
            addch('=' as chtype);
        }

        Ncurses {
            maxx: max_x,
            maxy: max_y,
        }
    }

    fn clear_input_line(&self) {

        for x in 0..self.maxx {
            mv(self.maxy - 1, x);
            addch(' ' as chtype);
        }
    }

    fn x(&self) -> i32 {
        let mut x = 0;
        let mut y = 0;
        getyx(stdscr, &mut y, &mut x);
        x
    }
}

impl InputOutput for Ncurses {

    fn quit(&self) {
        endwin();
    }

    fn println(&self, s: String, color: color::Color) {
        // TODO
    }

    fn read_line(&self) -> Option<String> {

        let mut buf: Vec<u8> = Vec::new();

        self.clear_input_line();
        mv(self.maxy - 1, 0);
        refresh();

        loop {
            refresh();
            let c = getch();

            match c as i32 {
                10 => { // TODO constant for enter
                    let s = String::from_utf8(buf.clone());
                    match s {
                        Ok(val) => { return Some(val); }
                        _ => { } // TODO
                    }
                }

                4 => { // TODO constant for ctrl d
                    return None;
                }

                127 => { // TODO constant for backspace
                    if self.x() > 0 {
                        mv(self.maxy - 1, self.x() - 1);
                        addch(' ' as chtype);
                        mv(self.maxy - 1, self.x() - 1);
                    }
                    if buf.len() > 0 {
                        buf.pop();
                    }
                }

                _ => {
                    addch(c as chtype);
                    buf.push(c as u8);
                }
            }

            if self.x() == self.maxx - 1 {
                self.clear_input_line();
                mv(self.maxy - 1, 0);
            }
        }

        None
    }
}

impl Callbacks for Ncurses { }

