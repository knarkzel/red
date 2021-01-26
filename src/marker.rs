#[derive(Debug, Default)]
pub struct Marker(pub usize, pub usize);

impl Marker {
    pub fn decrease_x(&mut self, value: usize) {
        self.0 = self.0.saturating_sub(value);
    }
    pub fn increase_x(&mut self, value: usize) {
        self.0 += value;
    }
    pub fn decrease_y(&mut self, value: usize) {
        self.1 = self.1.saturating_sub(value);
    }
    pub fn increase_y(&mut self, value: usize) {
        self.1 += value;
    }
    pub fn align_bounds(&mut self, offset: &Marker, len: (usize, usize)) {
        // horizontally >>>
        if self.0 >= len.0 && offset.0 == 0 {
            self.0 = len.0.saturating_sub(1);
        } else if (self.0 + offset.0) >= len.0 {
            self.decrease_x(1);
        }

        // vertically VVV
        if (self.1 + offset.1) >= len.1 {
            self.decrease_y(1);
        }
    }
}
