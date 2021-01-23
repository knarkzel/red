#[derive(Debug, PartialEq)]
pub enum Mode {
    Insert,
    Normal,
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

