use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
pub struct Row {
    pub text: String,
    pub len: usize,
}

impl Row {
    pub fn new() -> Row {
        Row {
            text: String::new(),
            len: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,  // insert mode
    Base,    // base mode
    Command, // command input mode
}

#[derive(Debug, Clone, Copy)]
pub enum CurMov {
    Up,
    Down,
    Left,
    Right,
    Goto(usize, usize),
}

pub fn read_file(name: &str, out: &mut Vec<String>) -> std::io::Result<()> {
    let file = std::fs::OpenOptions::new().read(true).open(name)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let text = line?;
        out.push(text);
    }
    Ok(())
}

pub fn write_file(name: &str, text: &Vec<String>) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(name)?;
    for r in text {
        file.write(r.as_bytes())?;
        file.write(b"\n")?;
    }
    file.flush()?;
    Ok(())
}
