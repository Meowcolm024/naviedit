pub struct State {
    indent: usize,
}

impl State {
    pub fn new() -> State {
        State { indent: 0 }
    }
}
