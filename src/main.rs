use std::io::{stdin, stdout};
use termion::{input::TermRead, raw::IntoRawMode};

mod lib;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let name = args.get(1).map(|n| n.clone());
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    let mut editor = lib::Editor::new(name, &mut stdout);

    editor.init();

    for c in stdin.keys() {
        editor.key_handle(&c.unwrap());
        editor.render();
    }
}
