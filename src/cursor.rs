#[derive(Debug)]
pub struct Cursor(pub u16, pub u16);

impl Cursor {
    pub fn new() -> Self {
        Self(1, 1)
    }
    pub fn move_left(&mut self) {
        if self.0 > 1 {
            self.0 -= 1;
        }
    }
    pub fn move_down(&mut self, offset: &(usize, usize), len: usize, line: Option<&String>) {
        if (self.1 as usize + offset.1) < len {
            self.1 += 1;
            if let Some(line) = line {
                if self.0 as usize > line.len() {
                    self.0 = line.len() as u16;
                }
            }
        }
    }
    pub fn move_up(&mut self, offset: &(usize, usize), line: Option<&String>) {
        if (self.1 as usize + offset.1) > 1 {
            self.1 -= 1;
            if let Some(line) = line {
                if self.0 as usize > line.len() {
                    self.0 = line.len() as u16;
                }
            }
        }
    }
    pub fn move_right(&mut self, offset: &(usize, usize), line: Option<&String>) {
        if let Some(line) = line {
            if (self.0 as usize + offset.0) < line.len() {
                self.0 += 1;
            }
        }
    }
}
