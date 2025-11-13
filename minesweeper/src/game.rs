use std::sync::{Arc, Mutex};

pub const BOMB: i8 = -1;
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
        }

        Arc::new(Mutex::new(game))
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

    pub fn reveal(&mut self) {
        let (i, j) = (self.current_y as usize, self.current_x as usize);
        match self.state[i][j] {
            STATE_HIDDEN => {
                if self.board[i][j] == BOMB {
                    self.game_over();
                    return;
                }
                self.state[i][j] = STATE_REVEALED
            },
            STATE_REVEALED => {},
            _ => {}
        };
    }

    pub fn toggle_flag(&mut self) {
        let (i, j) = (self.current_y as usize, self.current_x as usize);
        let isbomb = self.board[i][j] < 0;
        match self.state[i][j] {
            STATE_HIDDEN => {
                self.state[i][j] = 1;
                self.n_flagged += 1;
                if isbomb {
                    self.n_found += 1;
                }
            }
            STATE_FLAGGED => {
                self.state[i][j] = 0;
                self.n_flagged -= 1;
                if isbomb {
                    self.n_found -= 1;
                }
            }
            _ => return,
        };
    }
}
