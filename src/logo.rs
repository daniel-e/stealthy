extern crate term;

use std::io::prelude::*;

// font: doom
// http://www.network-science.de/ascii/
pub fn print_logo() {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::GREEN).unwrap();

    (write!(t, " _            _     _ _     _           _           _    \n")).unwrap();
    (write!(t, "(_)          (_)   (_) |   | |         | |         | |  \n")).unwrap();
    (write!(t, " _ _ ____   ___ ___ _| |__ | | ___  ___| |__   __ _| |_ \n")).unwrap();
    (write!(t, "| | '_ \\ \\ / / / __| | '_ \\| |/ _ \\/ __| '_ \\ / _` | __|\n")).unwrap();
    (write!(t, "| | | | \\ V /| \\__ \\ | |_) | |  __/ (__| | | | (_| | |_ \n")).unwrap();
    (write!(t, "|_|_| |_|\\_/ |_|___/_|_.__/|_|\\___|\\___|_| |_|\\__,_|\\__|\n\n" )).unwrap();

    t.reset().unwrap();
}
