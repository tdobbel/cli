use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::sudoku::{Sudoku, generate_puzzle, solve_at_most_twice};

pub struct Game {
    pub current_pos: (usize, usize),
    pub sudoku: Sudoku,
    pub sudoku_ref: Sudoku,
    pub notes: HashMap<(usize, usize), Vec<u8>>,
    pub game_state: GameState,
    pub board_state: [[u8; 9]; 9],
}

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub enum GameState {
    Normal,
    Note,
    Done,
}

impl Game {
    pub fn new() -> Arc<Mutex<Self>> {
        let empty_puzzle = Sudoku::empty();
        let mut game = Self {
            current_pos: (0, 0),
            sudoku: empty_puzzle.clone(),
            sudoku_ref: empty_puzzle,
            notes: HashMap::new(),
            game_state: GameState::Normal,
            board_state: [[0u8; 9]; 9],
        };

        game.reset(Difficulty::Medium);

        Arc::new(Mutex::new(game))
    }

    pub fn reset(&mut self, difficulty: Difficulty) {
        let n_clues = match difficulty {
            Difficulty::Easy => 36,
            Difficulty::Medium => 32,
            Difficulty::Hard => 25,
        };
        self.sudoku = generate_puzzle(n_clues);
        self.sudoku_ref = self.sudoku.clone();
        self.board_state = [[0u8; 9]; 9];
        for i in 0..9 {
            for j in 0..9 {
                if self.sudoku.grid[i][j] != 0 {
                    self.board_state[i][j] = 2;
                }
            }
        }
        self.notes.clear();
        self.game_state = GameState::Normal;
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
        if self.sudoku_ref.grid[i][j] != 0 {
            return;
        }
        match self.game_state {
            GameState::Normal => self.sudoku.grid[i][j] = number,
            GameState::Note => {
                let notes = self.notes.entry((i, j)).or_default();
                notes.push(number);
            }
            _ => {}
        }
    }

    pub fn toggle_notes(&mut self) {
        match self.game_state {
            GameState::Normal => self.game_state = GameState::Note,
            GameState::Note => self.game_state = GameState::Normal,
            _ => {}
        }
    }

    pub fn delete_number(&mut self) {
        let (i, j) = self.current_pos;
        if self.sudoku_ref.grid[i][j] != 0 {
            return;
        }
        match self.game_state {
            GameState::Normal => self.sudoku.grid[i][j] = 0,
            GameState::Note => {
                if let Some(notes) = self.notes.get_mut(&(i, j)) {
                    notes.pop();
                    if notes.is_empty() {
                        self.notes.remove(&(i, j));
                    }
                }
            }
            _ => {}
        }
    }

    pub fn check_solution(&mut self) {
        solve_at_most_twice(&mut self.sudoku_ref, &mut 0, true);
        for i in 0..9 {
            for j in 0..9 {
                if self.board_state[i][j] == 2 {
                    continue;
                }
                let num = self.sudoku.grid[i][j];
                let sol_num = self.sudoku_ref.grid[i][j];
                self.board_state[i][j] = if num == sol_num { 1 } else { 0 };
            }
        }
        self.game_state = GameState::Done;
    }

    pub fn solve(&mut self) {
        solve_at_most_twice(&mut self.sudoku_ref, &mut 0, true);
        for i in 0..9 {
            for j in 0..9 {
                if self.board_state[i][j] == 2 {
                    continue;
                }
                let num = self.sudoku.grid[i][j];
                let sol_num = self.sudoku_ref.grid[i][j];
                self.board_state[i][j] = if num == sol_num { 1 } else { 0 };
                self.sudoku.grid[i][j] = sol_num;
            }
        }
        self.game_state = GameState::Done;
    }

    pub fn update(&mut self) {
        if !matches!(self.game_state, GameState::Done) && self.sudoku.isfull() {
            self.check_solution();
        }
    }
}
