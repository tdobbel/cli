use std::sync::{Arc, Mutex};

use rand::Rng;

pub const MINE: i8 = -1;
pub const EMPTY: i8 = 0;
pub const STATE_HIDDEN: u8 = 0;
pub const STATE_FLAGGED: u8 = 1;
pub const STATE_REVEALED: u8 = 2;

pub enum GameState {
    Playing,
    ChangeLevel,
    GameOver,
}

pub enum Level {
    Beginner,
    Intermediate,
    Expert,
}

pub struct Game {
    pub level: Level,
    pub nx: u16,
    pub ny: u16,
    pub n_mines: u16,
    pub n_flagged: u16,
    pub n_found: u16,
    pub current_x: u16,
    pub current_y: u16,
    pub state: Vec<Vec<u8>>,
    pub board: Vec<Vec<i8>>,
    pub game_state: GameState,
}

impl Game {
    pub fn new() -> Arc<Mutex<Self>> {
        let mut game = Self {
            level: Level::Intermediate,
            nx: 0,
            ny: 0,
            n_mines: 0,
            n_found: 0,
            n_flagged: 10,
            current_x: 0,
            current_y: 0,
            state: Vec::new(),
            board: Vec::new(),
            game_state: GameState::Playing,
        };
        game.reset(Level::Intermediate);

        Arc::new(Mutex::new(game))
    }

    pub fn toggle_level_selection(&mut self) {
        match self.game_state {
            GameState::Playing => {
                self.game_state = GameState::ChangeLevel;
            }
            GameState::ChangeLevel => {
                self.game_state = GameState::Playing;
            }
            GameState::GameOver => {}
        }
    }

    pub fn select_level(&mut self, level: Level) {
        if matches!(self.game_state, GameState::Playing) {
            return;
        }
        self.reset(level);
    }

    fn reset(&mut self, level: Level) {
        self.level = level;
        self.n_found = 0;
        self.n_flagged = 0;
        match self.level {
            Level::Beginner => {
                self.nx = 9;
                self.ny = 9;
                self.n_mines = 10;
            }
            Level::Intermediate => {
                self.nx = 16;
                self.ny = 16;
                self.n_mines = 40;
            }
            Level::Expert => {
                self.nx = 30;
                self.ny = 16;
                self.n_mines = 99;
            }
        }
        self.state = vec![vec![STATE_HIDDEN; self.nx as usize]; self.ny as usize];
        self.board = vec![vec![EMPTY; self.nx as usize]; self.ny as usize];
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
        self.game_state = GameState::Playing;
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

    pub fn update(&mut self) {}

    pub fn game_over(&mut self) {
        self.game_state = GameState::GameOver;
    }

    fn clear_around(&mut self) {
        let (x, y) = (self.current_x as usize, self.current_y as usize);
        let n_flag = self.board[y][x];
        if n_flag == 0 {
            return;
        }
        let ix = self.current_x as i16;
        let iy = self.current_y as i16;
        let mut cntr_flag: i8 = 0;
        let mut todo_reveal: Vec<(usize, usize)> = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let px = ix + dx;
                let py = iy + dy;
                if px < 0 || px >= self.nx as i16 || py < 0 || py >= self.ny as i16 {
                    continue;
                }
                let x_ = px as usize;
                let y_ = py as usize;
                if self.state[y_][x_] == STATE_FLAGGED {
                    cntr_flag += 1
                } else if self.state[y_][x_] == STATE_HIDDEN {
                    todo_reveal.push((x_, y_));
                }
            }
        }
        if cntr_flag != n_flag {
            return;
        }
        todo_reveal.iter().for_each(|(px, py)| {
            let board_state = self.board[*py][*px];
            if board_state == MINE {
                self.game_over();
            }
            self.state[*py][*px] = STATE_REVEALED;
        });
    }

    fn reveal_recursive(&mut self, x: usize, y: usize) {
        self.state[y][x] = STATE_REVEALED;
        if self.board[y][x] != EMPTY {
            return;
        }
        let ix = x as i16;
        let iy = y as i16;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let px = ix + dx;
                let py = iy + dy;
                if px < 0 || px >= self.nx as i16 || py < 0 || py >= self.ny as i16 {
                    continue;
                }
                let x_ = px as usize;
                let y_ = py as usize;
                if self.state[y_][x_] == STATE_HIDDEN {
                    self.reveal_recursive(x_, y_);
                }
            }
        }
    }

    pub fn reveal(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
        let (x, y) = (self.current_x as usize, self.current_y as usize);
        match self.state[y][x] {
            STATE_HIDDEN => {
                if self.board[y][x] == MINE {
                    self.game_over();
                    return;
                }
                self.reveal_recursive(x, y);
            }
            STATE_REVEALED => self.clear_around(),
            _ => {}
        };
    }

    pub fn toggle_flag(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
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
