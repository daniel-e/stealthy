extern crate term;
extern crate ncurses;

use std::io;
use term::color;
use self::ncurses::*;

use humaninterface::InputOutput;
use callbacks::Callbacks;

pub struct Ncurses;

impl Ncurses {

    pub fn new() -> Ncurses {
        initscr();
        clear();
        noecho();
        //curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        refresh();
        Ncurses
    }
}

impl InputOutput for Ncurses {

    fn println(&self, s: String, color: color::Color) {
        // TODO
    }

    fn read_line(&self) -> Option<String> {

        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);
        mv(max_y - 1, 0);

        loop {
            let c = getch();
            match c {
                KEY_ENTER => {
                    mv(0, 0);
                    refresh();
                    return Some("bla".to_string());
                }

                _ => {
                    addch(c as chtype);
                    refresh();
                }
            }
        }

        None
    }
}

impl Callbacks for Ncurses { }

