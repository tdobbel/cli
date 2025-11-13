use std::{
    i8::MIN,
    sync::{Arc, Mutex},
};

use rand::Rng;

pub const MINE: i8 = -1;
pub const EMPTY: i8 = 0;
pub const STATE_HIDDEN: u8 = 0;
pub const STATE_FLAGGED: u8 = 1;
pub const STATE_REVEALED: u8 = 2;

pub struct Game {
    pub nx: u16,
    pub ny: u16,
    pub n_mines: u16,
    pub n_flagged: u16,
    pub n_found: u16,
    pub current_x: u16,
    pub current_y: u16,
    pub state: Vec<Vec<u8>>,
    pub board: Vec<Vec<i8>>,
}

impl Game {
    pub fn new() -> Arc<Mutex<Self>> {
        let nx = 9;
        let ny = 9;
        let state = vec![vec![STATE_HIDDEN; nx as usize]; ny as usize];
        let board = vec![vec![EMPTY; nx as usize]; ny as usize];
        let mut game = Self {
            nx,
            ny,
            n_mines: 10,
            n_found: 0,
            n_flagged: 10,
            current_x: 0,
            current_y: 0,
            state,
            board,
        };
        game.seed_mines();

        Arc::new(Mutex::new(game))
    }

    pub fn seed_mines(&mut self) {
        let mut n_seeded = 0;
        let mut rng = rand::rng();
        while n_seeded < self.n_mines {
            let x = rng.random_range(0..self.nx as usize);
            let y = rng.random_range(0..self.ny as usize);
            if self.board[y][x] == MINE {
                continue;
            }
            self.board[y][x] = MINE;
            let ix = x as i16;
            let iy = y as i16;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let x = ix + dx;
                    let y = iy + dy;
                    if x < 0 || x >= self.nx as i16 || y < 0 || y >= self.ny as i16 {
                        continue;
                    }
                    let board_state = self.board[y as usize].get_mut(x as usize).unwrap();
                    if *board_state != MINE {
                        *board_state += 1
                    }
                }
            }
            n_seeded += 1;
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

    pub fn update(&mut self) {
        return;
    }

    pub fn game_over(&mut self) {
        return;
    }

    fn reveal_recursive(&mut self, x: usize, y: usize) {}

    pub fn reveal(&mut self) {
        let (x, y) = (self.current_x as usize, self.current_y as usize);
        match self.state[y][x] {
            STATE_HIDDEN => {
                if self.board[y][x] == MINE {
                    self.game_over();
                    return;
                }
                self.reveal_recursive(x, y);
            }
            STATE_REVEALED => {}
            _ => {}
        };
    }

    pub fn toggle_flag(&mut self) {
        let (i, j) = (self.current_y as usize, self.current_x as usize);
        let ismine = self.board[i][j] == MINE;
        match self.state[i][j] {
            STATE_HIDDEN => {
                self.state[i][j] = STATE_FLAGGED;
                self.n_flagged += 1;
                if ismine {
                    self.n_found += 1;
                }
            }
            STATE_FLAGGED => {
                self.state[i][j] = STATE_HIDDEN;
                self.n_flagged -= 1;
                if ismine {
                    self.n_found -= 1;
                }
            }
            _ => (),
        };
    }
}
