#[derive(Debug)]
pub struct Cursor(pub u16, pub u16);

impl Default for Cursor {
    fn default() -> Self {
        Self(1, 1)
    }
}

impl Cursor {
    pub fn move_left(&mut self, offset: &(usize, usize)) {
        if (self.0 as usize + offset.0) > 1 {
            self.0 -= 1;
        }
    }
    pub fn move_down(&mut self, offset: &(usize, usize), amount_lines: usize, len: usize) {
        if (self.1 as usize + offset.1) < amount_lines {
            self.1 += 1;
            self.align_bounds(len);
        }
    }
    pub fn move_up(&mut self, offset: &(usize, usize), len: usize) {
        if (self.1 as usize + offset.1) > 1 {
            self.1 -= 1;
            self.align_bounds(len);
        }
    }
    pub fn move_right(&mut self, offset: &(usize, usize), len: usize) {
        if (self.0 as usize + offset.0) < len {
            self.0 += 1;
        }
    }
    fn align_bounds(&mut self, len: usize) {
        if len == 0 {
            self.0 = 1;
        } else if self.0 as usize > len {
            self.0 = len as u16;
        }
    }
}
