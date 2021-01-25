#[derive(Debug, PartialEq)]
pub enum Mode {
    Insert,
    Normal,
    Command,
    Spatial(char),
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

