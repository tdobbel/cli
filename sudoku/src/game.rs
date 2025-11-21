use std::sync::{Arc, Mutex};

use crate::sudoku::{Sudoku, generate_puzzle};

pub struct Game {
    pub current_pos: (usize, usize),
    pub sudoku: Sudoku,
    pub sudoku_ref: Sudoku,
}

impl Game {
    pub fn new() -> Arc<Mutex<Self>> {
        let sudo = generate_puzzle(25);
        let sudo_sol = sudo.clone();
        let game = Self {
            current_pos: (0, 0),
            sudoku: sudo,
            sudoku_ref: sudo_sol,
        };

        Arc::new(Mutex::new(game))
    }

    pub fn move_up(&mut self) {
        let current_y = self.current_pos.0;
        match current_y.checked_sub(1) {
            Some(y) => self.current_pos.0 = y,
            None => self.current_pos.0 = 8,
        };
    }

    pub fn move_down(&mut self) {
        self.current_pos.0 = (self.current_pos.0 + 1) % 9;
    }

    pub fn move_left(&mut self) {
        let current_x = self.current_pos.1;
        match current_x.checked_sub(1) {
            Some(x) => self.current_pos.1 = x,
            None => self.current_pos.1 = 8,
        };
    }

    pub fn move_right(&mut self) {
        self.current_pos.1 = (self.current_pos.1 + 1) % 9;
    }

    pub fn set_number(&mut self, number: u8) {
        let (i, j) = self.current_pos;
        if self.sudoku_ref.grid[i][j] == 0 {
            self.sudoku.grid[i][j] = number;
        }
    }

    pub fn delete_number(&mut self) {
        let (i, j) = self.current_pos;
        if self.sudoku_ref.grid[i][j] == 0 {
            self.sudoku.grid[i][j] = 0;
        }
    }
}
