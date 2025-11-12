pub const NROWS: u16 = 9;
pub const NCOLS: u16 = 9;

pub struct Game {
    nx: usize,
    ny: usize,
    current_x: usize,
    current_y: usize,
}

impl Game {
    pub fn new() -> Self {
        Self {
            nx: NCOLS as usize,
            ny: NROWS as usize,
            current_x: 0,
            current_y: 0,
        }
    }
    pub fn move_left(&mut self) {
        match self.current_x.checked_sub(1) {
            Some(x) => self.current_x = x,
            None => self.current_x = self.nx - 1,
        };
    }

    pub fn move_right(&mut self) {
        self.current_x = (self.current_x + 1) % self.nx;
    }

    pub fn move_up(&mut self) {
        match self.current_y.checked_sub(1) {
            Some(y) => self.current_y = y,
            None => self.current_y = self.ny - 1,
        };
    }

    pub fn move_down(&mut self) {
        self.current_y = (self.current_y + 1) % self.ny;
    }
}
