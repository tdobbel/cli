use std::sync::{Arc, Mutex, mpsc};
use std::usize;

use rand::Rng;

pub const BOARD_WIDTH: u16 = 12;
pub const BOARD_HEIGHT: u16 = 24;
pub const COLORS: [u8; 4] = [1, 2, 3, 4];

type GameBoard = [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

#[derive(Copy, Clone)]
pub enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoType {
    pub fn random() -> Self {
        use TetrominoType::*;
        let types = [I, O, T, S, Z, J, L];
        let mut rng = rand::rng();
        let idx = rng.random_range(0..types.len());
        types[idx]
    }
}

fn get_random_color() -> u8 {
    let mut rng = rand::rng();
    let indx = rng.random_range(0..COLORS.len());
    COLORS[indx]
}

pub enum Direction {
    Left,
    Right,
    Down,
}

pub enum GameState {
    Running,
    Paused,
    GameOver,
}

// pub enum MoveResult {
//     Moved,
//     Collided,
//     OutOfBounds,
//     Spawned,
//     NoTetromino,
// }

#[derive(Debug)]
pub struct Game {
    pub current_mino: Option<Tetromino>,
    pub board: GameBoard,
}

#[derive(Debug)]
pub struct Tetromino {
    pub urcrnr_x: i16,
    pub urcrnr_y: i16,
    box_size: usize,
    pub pixels: [(i16, i16); 4],
    pub color: u8,
}

impl Tetromino {
    fn new(tetromino_type: TetrominoType, color: u8) -> Self {
        let xmax = BOARD_WIDTH as i16;
        let (box_size, pixels) = match tetromino_type {
            TetrominoType::I => (4, [(0, 1), (1, 1), (2, 1), (3, 1)]),
            TetrominoType::J => (3, [(0, 0), (0, 1), (1, 1), (2, 1)]),
            TetrominoType::L => (3, [(0, 1), (1, 1), (2, 1), (2, 0)]),
            TetrominoType::O => (2, [(0, 0), (1, 0), (1, 1), (0, 1)]),
            TetrominoType::S => (3, [(0, 1), (1, 1), (1, 0), (2, 0)]),
            TetrominoType::T => (3, [(0, 1), (1, 1), (1, 0), (2, 1)]),
            TetrominoType::Z => (3, [(0, 0), (1, 0), (1, 1), (2, 1)]),
        };
        let urcrnr_x = (xmax - box_size as i16) / 2;
        Self {
            urcrnr_x,
            urcrnr_y: 0,
            box_size,
            pixels,
            color,
        }
    }

    pub fn shift(&mut self, direction: Direction, board: &mut GameBoard) {
        let xmax = BOARD_WIDTH as i16;
        let ymax = BOARD_HEIGHT as i16;
        let (xo, yo) = match direction {
            Direction::Left => (self.urcrnr_x - 1, self.urcrnr_y),
            Direction::Right => (self.urcrnr_x + 1, self.urcrnr_y),
            Direction::Down => (self.urcrnr_x, self.urcrnr_y + 1),
        };
        for (px, py) in self.pixels.iter() {
            let ix = xo + *px;
            let iy = yo + *py;
            if ix < 0 || ix >= xmax || iy < 0 || iy >= ymax {
                return;
            }
            if board[iy as usize][ix as usize] > 0 {
                return;
            }
        }
        self.urcrnr_x = xo;
        self.urcrnr_y = yo;
    }

    pub fn rotate(&mut self, clockwise: bool, board: &mut GameBoard) {
        let box_size = self.box_size as i16;
        let xo = box_size / 2;
        let yo = box_size / 2;
        let xmax = BOARD_WIDTH as i16;
        let ymax = BOARD_HEIGHT as i16;
        let mut rotated_pixels: [(i16, i16); 4] = [(0, 0); 4];
        let mut shift_left = false;
        let mut shift_up = false;
        let par_size = self.box_size % 2 == 0;
        for (i, (px, py)) in self.pixels.iter().enumerate() {
            let mut x = *px;
            let mut y = *py;
            if par_size && x >= xo {
                x += 1;
            }
            if par_size && y >= yo {
                y += 1;
            }
            let (mut xr, mut yr) = if clockwise {
                (xo + (y - yo), yo - (x - xo))
            } else {
                (xo - (y - yo), yo + (x - xo))
            };
            if par_size && xr >= xo {
                xr -= 1;
            }
            if par_size && yr >= yo {
                yr -= 1;
            }
            shift_left = shift_left || (xr >= box_size);
            shift_up = shift_up || (yr >= box_size);
            rotated_pixels[i] = (xr, yr);
        }
        if shift_up {
            rotated_pixels.iter_mut().for_each(|xy| xy.1 -= 1);
        }
        if shift_left {
            rotated_pixels.iter_mut().for_each(|xy| xy.0 -= 1);
        }
        for (x, y) in rotated_pixels.iter() {
            let ix = self.urcrnr_x + *x;
            let iy = self.urcrnr_y + *y;
            if ix < 0 || iy < 0 || iy >= ymax || ix >= xmax {
                return;
            }
            if board[iy as usize][ix as usize] != 0 {
                return;
            }
        }
        self.pixels = rotated_pixels;
    }
}

impl Game {
    pub fn new() -> Arc<Mutex<Self>> {
        let mut game = Self {
            current_mino: None,
            board: Default::default(),
        };
        game.spawn_tetromino();

        let game_state = Arc::new(Mutex::new(game));
        game_state
    }

    pub fn spawn_tetromino(&mut self) {
        let tetromino = Tetromino::new(TetrominoType::random(), get_random_color());
        for (px, py) in tetromino.pixels.iter() {
            let x = tetromino.urcrnr_x + *px;
            let y = tetromino.urcrnr_y + *py;
            if self.board[y as usize][x as usize] != 0 {
                return;
            }
        }
        self.current_mino = Some(tetromino);
    }

    pub fn rotate(&mut self) {
        if let Some(mino) = &mut self.current_mino {
            mino.rotate(true, &mut self.board);
        }
    }

    pub fn move_left(&mut self) {
        if let Some(mino) = &mut self.current_mino {
            mino.shift(Direction::Left, &mut self.board);
        }
    }

    pub fn move_right(&mut self) {
        if let Some(mino) = &mut self.current_mino {
            mino.shift(Direction::Right, &mut self.board);
        }
    }

    pub fn move_down(&mut self) {
        if let Some(mino) = &mut self.current_mino {
            mino.shift(Direction::Down, &mut self.board);
        }
    }
}
