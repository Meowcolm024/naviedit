use std::io::Stdout;
use termion::{event::Key, raw::RawTerminal};

use super::{
    lib::{read_file, write_file, CurMov, Mode, Row},
    state::State,
    view::View,
};

pub struct Editor<'a> {
    mode: Mode,
    buffer: Vec<String>,
    cmd: Vec<char>,
    focus: (usize, usize),
    name: Option<&'a str>,
    view: View<'a>,
    state: State,
}

impl Editor<'_> {
    pub fn new<'a>(name: Option<&'a str>, stdout: &'a mut RawTerminal<Stdout>) -> Editor<'a> {
        let size = termion::terminal_size().unwrap();
        let mut buffer = Vec::new();
        if let Some(n) = name {
            read_file(n, &mut buffer).unwrap_or_else(|_| {
                buffer.clear();
                buffer.push(String::new())
            });
        } else {
            buffer.push(String::new());
        }
        Editor {
            mode: Mode::Base,
            buffer,
            cmd: Vec::new(),
            focus: (0, 0),
            name,
            view: View::new(stdout, (size.0 as usize, size.1 as usize)),
            state: State::new(),
        }
    }

    pub fn init(&mut self) {
        self.clear();
        self.render();
    }

    fn clear(&mut self) {
        self.view.clear().unwrap();
    }

    pub fn render(&mut self) {
        self.view
            .render(self.mode, &self.focus, &self.buffer, &self.cmd)
            .unwrap();
    }

    fn update_focus(&mut self, dir: CurMov) {
        match dir {
            CurMov::Up => {
                if self.focus.1 > 0 {
                    self.focus.1 -= 1;
                    if self.focus.0 >= self.buffer[self.focus.1].len() {
                        self.focus.0 = self.buffer[self.focus.1].len();
                    }
                }
            }
            CurMov::Down => {
                if self.focus.1 < self.buffer.len() - 1 {
                    self.focus.1 += 1;
                    if self.focus.0 >= self.buffer[self.focus.1].len() {
                        self.focus.0 = self.buffer[self.focus.1].len();
                    }
                }
            }
            CurMov::Left => {
                if self.focus.0 > 0 {
                    self.focus.0 -= 1;
                }
            }
            CurMov::Right => {
                if self.focus.0 < self.buffer[self.focus.1].len() {
                    self.focus.0 += 1;
                }
            }
            CurMov::Goto(x, y) => {
                assert!(y < self.buffer.len() && x <= self.buffer[y].len());
                self.focus.0 = x;
                self.focus.1 = y;
            }
        }
        self.view.focus_to_cursor(self.focus);
    }

    pub fn key_handle(&mut self, key: &Key) {
        match self.mode {
            Mode::Base => self.base_mode(key),
            Mode::Insert => self.insert_mode(key),
            Mode::Command => self.command_mode(key),
        }
    }

    fn base_mode(&mut self, key: &Key) {
        match key {
            Key::Char('i') => self.mode = Mode::Insert,
            Key::Char(':') => self.mode = Mode::Command,
            Key::Up => self.update_focus(CurMov::Up),
            Key::Down => self.update_focus(CurMov::Down),
            Key::Left => self.update_focus(CurMov::Left),
            Key::Right => self.update_focus(CurMov::Right),
            _ => (),
        }
    }

    fn insert_mode(&mut self, key: &Key) {
        match key {
            Key::Esc => self.mode = Mode::Base,
            Key::Up => self.update_focus(CurMov::Up),
            Key::Down => self.update_focus(CurMov::Down),
            Key::Left => self.update_focus(CurMov::Left),
            Key::Right => self.update_focus(CurMov::Right),
            Key::Delete | Key::Backspace => {
                if self.focus.0 > 0 {
                    self.buffer[self.focus.1].replace_range(self.focus.0 - 1..self.focus.0, "");
                    self.update_focus(CurMov::Left);
                } else if self.focus.0 == 0 && self.focus.1 > 0 {
                    let cur = self.buffer[self.focus.1].clone();
                    self.buffer[self.focus.1 - 1] += cur.as_str();
                    self.update_focus(CurMov::Goto(
                        self.buffer[self.focus.1 - 1].len() - cur.len(),
                        self.focus.1 - 1,
                    ));
                    self.buffer.remove(self.focus.1 + 1);
                }
            }
            Key::Char('\n') => {
                let row = self.buffer[self.focus.1].clone();
                self.buffer[self.focus.1].truncate(self.focus.0);
                let new_row = String::from(&row[self.focus.0..row.len()]);
                self.buffer.insert(self.focus.1 + 1, new_row);
                self.update_focus(CurMov::Goto(0, self.focus.1 + 1));
            }
            Key::Char(c) => {
                self.buffer[self.focus.1].insert(self.focus.0, *c);
                self.update_focus(CurMov::Right);
            }
            _ => (),
        }
    }

    fn command_mode(&mut self, key: &Key) {
        match key {
            Key::Char('\n') => {
                let cmd_str: String = self.cmd.iter().collect();
                let mut cmd: Vec<&str> = cmd_str.split(' ').rev().collect();
                if let Some(x) = cmd.pop() {
                    match x {
                        "q" => {
                            self.clear();
                            std::process::exit(0);
                        }
                        "w" => {
                            if let Some(file) = cmd.pop() {
                                write_file(file, &self.buffer).unwrap();
                            } else if let Some(file) = self.name {
                                write_file(file, &self.buffer).unwrap();
                            }
                        }
                        _ => (),
                    }
                }
                self.cmd.clear();
                self.mode = Mode::Base;
            }
            Key::Char(c) => self.cmd.push(*c),
            Key::Backspace | Key::Delete => {
                self.cmd.pop();
            }
            Key::Esc => {
                self.cmd.clear();
                self.mode = Mode::Base
            }
            _ => (),
        }
    }
}
