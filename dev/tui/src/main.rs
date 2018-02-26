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
use std::iter::repeat;

struct App {
    size: Rect,
    tabs: MyTabs,
    input: String,
    messages: Vec<String>,
    line_offset: i32,
    selected_contact: usize,
    contacts: Vec<String>,
    options: Vec<String>,
    selected_option: usize,
    options_content: Options,
}

struct MyTabs {
    pub titles: Vec<String>,
    pub selection: usize,
}

struct Options {
    interface: String,
    add_contact: AddContact,
}

struct AddContact {
    name: String,
    ip: String,
    key: String,
}

impl MyTabs {
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

fn create_app() -> App {
    App {
        size: Rect::default(),
        tabs: MyTabs {
            titles: vec![String::from("Messages"), String::from("Options")],
            selection: 0
        },
        input: String::new(),
        messages: vec![String::new(), String::new()],
        line_offset: 0,
        selected_contact: 0,
        contacts: vec![String::from("a")],
        options: vec![String::from("Set network interface "), String::from("Add contact          ")],
        selected_option: 0,
        options_content: Options {
            interface: String::new(),
            add_contact: AddContact {
                name: String::new(),
                ip: String::new(),
                key: String::new(),
            }
        }
    }
}

fn main() {
    let backend = MouseBackend::new().unwrap();
    let mut term = Terminal::new(backend).unwrap();

    let mut app = create_app();

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
            event::Key::Esc => {
                break;
            },
            _ => {}
        }

        if app.tabs.selection == 0 { // Messages
            match evt {
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
                event::Key::Backspace => {
                    app.input.pop();
                },
                event::Key::Char(c) => {
                    app.input.push(c);
                },
                _ => {}
            }
        } else if app.tabs.selection == 1 { // Options
            match evt {
                event::Key::Char('\n') => {

                },
                event::Key::Down => {
                    if app.selected_option < app.options.len() - 1 {
                        app.selected_option += 1;
                    }
                },
                event::Key::Up => {
                    if app.selected_option > 0 {
                        app.selected_option -= 1;
                    }
                }
                _ => {}
            }
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

fn draw(t: &mut Terminal<MouseBackend>, app: &mut App) {
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
                0 => { add_messages_widget(t, &chunks[1], &chunks[2], app); }
                1 => { add_options_widget(t, &chunks[1], app); }
                _ => {}
            }
        });
    t.draw().unwrap()
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

fn add_messages_widget(t: &mut Terminal<MouseBackend>, chunk1: &Rect, chunk2: &Rect, app: &mut App) {
    let siz = t.size().unwrap();
    let app_size = app.size.clone();

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
        .render(t, chunk1, |t, chunks| {
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
        });

    let mut s = app.input.clone();
    s.push('▄');
    let slen = s.chars().count();
    let win_width = (siz.width - 2) as usize;
    if slen > win_width {
        s = s.chars().skip(slen - win_width).collect();
    }

    Paragraph::default()
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Your message "))
        .text(&s)
        .render(t, chunk2);
}

fn add_options_widget(t: &mut Terminal<MouseBackend>, chunk: &Rect, app: &mut App) {
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Fixed(25), Size::Min(0)])
        .render(t, chunk, |t, chunks| {
            SelectableList::default()
                .block(Block::default().title(" Options ").borders(Borders::ALL))
                .items(&app.options)
                .select(app.selected_option)
                .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Gray).modifier(Modifier::Bold))
                .render(t, &chunks[0]);
            Block::default().borders(Borders::ALL).render(t, &chunks[1]);
            match app.selected_option {
                0 => { add_network_interface_widget(t, &chunks[1]); },
                1 => { add_contact_widget(t, &chunks[1]); },
                _ => {}
            }
        });
}

fn add_network_interface_widget(t: &mut Terminal<MouseBackend>, chunk: &Rect) {
    Group::default()
        .direction(Direction::Vertical)
        .margin(2)
        .sizes(&[Size::Fixed(1)])
        .render(t, chunk, |t, chunks| {
            add_textfield("Interface:",12, 10, t, &chunks[0]);
        });
}

fn add_contact_widget(t: &mut Terminal<MouseBackend>, chunk: &Rect) {
    Group::default()
        .direction(Direction::Vertical)
        .margin(2)
        .sizes(&[Size::Fixed(1), Size::Fixed(1), Size::Fixed(1)])
        .render(t, chunk, |t, chunks| {
            add_textfield("Name:", 8, 15, t, &chunks[0]);
            add_textfield("IP:", 8,15, t, &chunks[1]);
            add_textfield("Key:", 8,32, t, &chunks[2]);
        });
}

fn add_textfield(label: &str, label_length:usize, input_length: usize, t: &mut Terminal<MouseBackend>, chunk: &Rect) {
    let f: String = repeat('_').take(input_length).collect();

    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Fixed(label_length as u16), Size::Min(0)])
        .render(t, chunk, |t, chunks| {
            Paragraph::default()
                .style(Style::default().fg(Color::Yellow))
                .text(label)
                .render(t, &chunks[0]);
            Paragraph::default()
                .style(Style::default().fg(Color::Yellow))
                .text(&f)
                .render(t, &chunks[1]);
        });
}
