use std::default;

pub const BOARD_WIDTH: u16 = 12;
pub const BOARD_HEIGHT: u16 = 24;

type GameBoard = [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

pub enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

pub enum Direction {
    Left,
    Right,
    Down,
}

pub struct TetrisBoard {
    tetromino: Option<Tetromino>;
    cells: GameBoard 
}

pub struct Tetromino {
    urcrnr_x: usize,
    urcrnr_y: usize,
    box_size: usize,
    pixels: Vec<(usize, usize)>,
}

impl Tetromino {
    fn new(tetromino_type: TetrominoType) -> Self {
        match tetromino_type {
            TetrominoType::I => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 4,
                pixels: vec![(0, 1), (1, 1), (2, 1), (3, 1)],
            },
            TetrominoType::J => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 0), (0, 1), (1, 1), (2, 1)],
            },
            TetrominoType::L => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 1), (1, 1), (2, 1), (2, 0)],
            },
            TetrominoType::O => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 2,
                pixels: vec![(0, 0), (1, 0), (1, 1), (0, 1)],
            },
            TetrominoType::S => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 1), (1, 1), (1, 0), (2, 0)],
            },
            TetrominoType::T => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 1), (1, 1), (1, 0), (2, 1)],
            },
            TetrominoType::Z => Self {
                urcrnr_x: 0,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 0), (1, 0), (1, 1), (2, 1)],
            },
        }
    }

    pub fn deactivate_cells(self, board: &mut GameBoard) {
        for (px, py) in self.pixels.iter() {
            let x = self.urcrnr_x + *px;
            let y = self.urcrnr_y + *py;
            board[y][x] = 0;
        }
    }

    pub fn shift(&mut self, direction: Direction, board: &mut GameBoard) {
    }

    pub fn rotate(&mut self, clockwise: bool, board: &mut GameBoard) {
        let xo = self.box_size / 2;
        let yo = self.box_size / 2;
        let mut rotated_pixels = Vec::new();
        let mut shift_left = false;
        let mut shift_up = false;
        let par_size = self.box_size % 2 == 0;
        for (px, py) in self.pixels.iter() {
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
            shift_left = shift_left || (xr >= self.box_size);
            shift_up = shift_up || (yr >= self.box_size);
            rotated_pixels.push((xr, yr))
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
            if board[iy][ix] > 0 {
                return;
            }
        }
        self.pixels = rotated_pixels;
    }
}

impl TetrisBoard {
    pub fn new() -> Self {
        Self { tetromino: None, cells: Default::default() }
    }


}
