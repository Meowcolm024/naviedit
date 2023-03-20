use super::lib::{CurMov, Mode};
use std::{
    cmp::{self},
    io::{Stdout, Write},
};
use termion::raw::RawTerminal;

pub struct View<'a> {
    stdout: &'a mut RawTerminal<Stdout>,
    size: (usize, usize),   // size of the terminal
    cursor: (usize, usize), // cursor position
    full_update: bool,
}

impl View<'_> {
    pub fn new<'a>(stdout: &'a mut RawTerminal<Stdout>, size: (usize, usize)) -> View<'a> {
        View {
            stdout,
            size,
            cursor: (1, 2),
            full_update: false,
        }
    }

    pub fn focus_to_cursor(&mut self, focus: (usize, usize)) {
        if focus.1 >= self.size.1 - 1 {
            self.cursor.1 = self.size.1
        } else {
            self.cursor.1 = focus.1 + 2
        }
        if focus.0 >= self.size.0 {
            self.cursor.0 = self.size.0
        } else {
            self.cursor.0 = focus.0 + 1
        }
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        write!(self.stdout, "{}", termion::clear::All)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn render_line(&mut self, focus: &(usize, usize), buffer: &String) -> std::io::Result<()> {
        if self.cursor.0 != self.size.0 {
            let l = cmp::min(buffer.len(), self.size.0);
            write!(
                self.stdout,
                "{}{}{}",
                termion::cursor::Goto(1, self.cursor.1 as u16),
                termion::clear::CurrentLine,
                &buffer[0..l],
            )?;
        } else {
            if buffer.len() + self.size.0 <= focus.0 {
                // when case line too short
                write!(
                    self.stdout,
                    "{}{}",
                    termion::cursor::Goto(1, self.cursor.1 as u16),
                    termion::clear::CurrentLine,
                )?;
            } else {
                let x0 = focus.0 + 1 - self.size.0;
                let x1 = cmp::min(focus.0 + 1, buffer.len());
                write!(
                    self.stdout,
                    "{}{}{}",
                    termion::cursor::Goto(1, self.cursor.1 as u16),
                    termion::clear::CurrentLine,
                    &buffer[x0..x1],
                )?;
            }
        }
        Ok(())
    }

    fn render_text(&mut self, focus: &(usize, usize), buffer: &Vec<String>) -> std::io::Result<()> {
        let cy = self.cursor.1;
        assert!(cy >= 2);

        if self.cursor.1 != self.size.1 {
            self.cursor.1 = 2;
            for i in 0..(cmp::min(buffer.len(), self.size.1 - 1)) {
                self.render_line(focus, &buffer[i])?;
                self.cursor.1 += 1;
            }
        } else {
            self.cursor.1 = 2;
            for i in (focus.1 + 2 - self.size.1)..(focus.1 + 1) {
                self.render_line(focus, &buffer[i])?;
                self.cursor.1 += 1;
            }
        }

        self.cursor.1 = cy;
        write!(
            self.stdout,
            "{}",
            termion::cursor::Goto(self.cursor.0 as u16, self.cursor.1 as u16)
        )?;
        Ok(())
    }

    fn render_command(&mut self, buffer: &Vec<char>) -> std::io::Result<()> {
        write!(
            self.stdout,
            "{}{}COMMAND MODE:{}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine,
            buffer.iter().collect::<String>()
        )?;
        Ok(())
    }

    pub fn render(
        &mut self,
        mode: Mode,
        focus: &(usize, usize),
        txt_buffer: &Vec<String>,
        cmd_buffer: &Vec<char>,
    ) -> std::io::Result<()> {
        // render text
        self.render_text(focus, txt_buffer)?;
        // render header
        match mode {
            Mode::Insert => write!(
                self.stdout,
                "{}{}INSERT MODE{}",
                termion::cursor::Goto(1, 1),
                termion::clear::CurrentLine,
                termion::cursor::Goto(self.cursor.0 as u16, self.cursor.1 as u16)
            )?,
            Mode::Base => write!(
                self.stdout,
                "{}{}BASE MODE{:?}{:?}{}",
                termion::cursor::Goto(1, 1),
                termion::clear::CurrentLine,
                self.cursor,
                self.size,
                termion::cursor::Goto(self.cursor.0 as u16, self.cursor.1 as u16)
            )?,
            Mode::Command => self.render_command(cmd_buffer)?,
        }
        // flush
        self.stdout.flush()?;
        Ok(())
    }
}
