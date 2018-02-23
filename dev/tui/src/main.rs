extern crate tui;
extern crate termion;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::{Rect, Direction, Size, Group};
use tui::style::{Color, Style, Modifier};
use tui::widgets::{Block, Widget, Borders, Tabs, Paragraph, SelectableList};

use termion::input::TermRead;
use termion::event;

use std::io;
use std::cmp;

struct App<'a> {
    size: Rect,
    tabs: MyTabs<'a>,
    input: String,
    messages: Vec<String>,
    line_offset: i32,
    selected_contact: usize,
    contacts: Vec<&'a str>,
}

struct MyTabs<'a> {
    pub titles: Vec<&'a str>,
    pub selection: usize,
}

impl<'a> MyTabs<'a> {
    pub fn next(&mut self) {
        self.selection = (self.selection + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        } else {
            self.selection = self.titles.len() - 1;
        }
    }
}

fn main() {
    let backend = MouseBackend::new().unwrap();
    let mut term = Terminal::new(backend).unwrap();

    let mut app = App {
        size: Rect::default(),
        tabs: MyTabs {
            titles: vec!["Messages", "Options"],
            selection: 0
        },
        input: String::new(),
        messages: vec![String::new(), String::new()],
        line_offset: 0,
        selected_contact: 0,
        contacts: vec!["abcd", "efg"],
    };

    term.clear().unwrap();
    term.hide_cursor().unwrap();
    app.size = term.size().unwrap();

    draw(&mut term, &mut app);

    let stdin = io::stdin();
    for c in stdin.keys() {
        let evt = c.unwrap();
        match evt {
            // https://docs.rs/termion/1.5.1/termion/event/enum.Key.html
            event::Key::Left => {
                app.tabs.previous();
            },
            event::Key::Right => {
                app.tabs.next();
            },
            event::Key::Char('\n') => {
                append_message(&mut app.messages[app.selected_contact], &app.input);
                app.input.clear();
                app.line_offset = 0;
            },
            event::Key::Down => {
                if app.line_offset < 0 {
                    app.line_offset += 1;
                }
            },
            event::Key::Up => {
                app.line_offset -= 1;
            },
            event::Key::PageDown => {
                app.selected_contact = cmp::min(app.contacts.len() - 1, app.selected_contact + 1)
            }
            event::Key::PageUp => {
                if app.selected_contact > 0 {
                    app.selected_contact -= 1;
                }
            }
            event::Key::Esc => {
                break;
            },
            event::Key::Backspace => {
                app.input.pop();
            },
            event::Key::Char(c) => {
                app.input.push(c);
            },
            _ => {}
        }

        draw(&mut term, &mut app);
    }

    term.show_cursor().unwrap();
}

fn append_message(buf: &mut String, msg: &str) {
    // limit to maximum of 1000 lines
    loop {
        let n = buf.split("\n").count();
        if n < 1000 {
            break;
        }
        let s: String = buf.chars().skip_while(|x| *x != '\n').skip(1).collect();
        buf.clear();
        buf.push_str(&s);
    }
    if buf.len() > 0 {
        buf.push('\n');
    }
    buf.push_str(msg);
}

fn number_of_lines_in_window(buf: &String, w: u16, h: u16, skp: usize) -> u16 {
    let lines = buf.split("\n").collect::<Vec<_>>();
    let mut c = 0;
    let mut lc = 0;
    for l in lines.iter().skip(skp) {
        lc += 1;
        let mut n = l.chars().count(); // number of characters for current line
        if lc < lines.len() as u16{
            n += 1;  // if not last line add n++ because it contains a newline
        }
        while n > 0 {
            c += 1;
            n -= cmp::min(n, w as usize);
        }
    }
    c
}

fn draw(t: &mut Terminal<MouseBackend>, app: &mut App) {
    let siz = t.size().unwrap();

    let app_size = app.size.clone();

    Group::default()
        .direction(Direction::Vertical)
        .margin(0)
        .sizes(&[Size::Fixed(3), Size::Min(0), Size::Fixed(3)])
        .render(t, &app_size, |t, chunks| {

            Tabs::default()
                .block(Block::default().borders(Borders::ALL))
                .titles(&app.tabs.titles)
                .select(app.tabs.selection)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(t, &chunks[0]);

            match app.tabs.selection {
                0 => {
                    let h = siz.height - 8;
                    let w = siz.width - 2;
                    let k = app.messages[app.selected_contact].split("\n").count() as u16;
                    let mut s: i32 = 0;
                    if k > h {
                        s = (k - h) as i32;
                        while number_of_lines_in_window(&app.messages[app.selected_contact], w, h, s as usize) > h {
                            s += 1;
                        }
                    }
                    s += app.line_offset;
                    if s < 0 {
                        app.line_offset -= s;
                        s = 0;
                    }

                    Group::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .sizes(&[Size::Min(0), Size::Fixed(15)])
                        .render(t, &chunks[1], |t, chunks| {
                            // Messages
                            let mut title = String::from(format!(" {} ", app.contacts[app.selected_contact]));
                            Paragraph::default()
                                .style(Style::default())
                                .block(Block::default().borders(Borders::ALL).title(&title))
                                .text(&app.messages[app.selected_contact])
                                .scroll(s as u16)
                                .wrap(true)
                                .render(t, &chunks[0]);

                            SelectableList::default()
                                .block(Block::default().title(" Contacts ").borders(Borders::ALL))
                                .items(&app.contacts)
                                .select(app.selected_contact)
                                .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::Bold))
                                .highlight_symbol("▶")
                                .render(t, &chunks[1]);
                        })
                }
                1 => {
                }
                _ => {}
            }


            let mut s = app.input.clone();
            s.push('▂');
            let slen = s.chars().count();
            let win_width = (siz.width - 2) as usize;
            if slen > win_width {
                s = s.chars().skip(slen - win_width).collect();
            }

            Paragraph::default()
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(" Your message "))
                .text(&s)
                .render(t, &chunks[2]);
        });


    t.draw().unwrap()
}