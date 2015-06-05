extern crate term;
extern crate ncurses;

use std::io;
use term::color;
use self::ncurses::*;

use humaninterface::InputOutput;
use callbacks::Callbacks;

struct Ncurses {
    r_x: i32,
    r_y: i32,
    w_x: i32,
    w_y: i32
}

impl Ncurses {

    fn new() -> Ncurses {
        initscr();
        clear();
        refresh();
        
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);
        mv(max_y, 0);

        Ncurses {
            r_x: 0,
            r_y: 0,
            w_x: 0,
            w_y: max_y
        }
    }

    fn print(&self, x: i32, y: i32, s: String) {

        mv(y, x);
        printw(&s);
        refresh();
    }

    fn read_line(&self, s: &mut String) -> i32 {
        getch();
        s.push_str("bla");
        1
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
        mv(max_y, 0);

        loop {
            let c = getch();
            addch(c as chtype);
            refresh();
        }
    }
}

impl Callbacks for Ncurses { }

