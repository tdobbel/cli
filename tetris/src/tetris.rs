use rand::Rng;

pub const BOARD_WIDTH: u16 = 12;
pub const BOARD_HEIGHT: u16 = 24;
pub const N_COLOR: u8 = 4;

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
    rng.random_range(1..=N_COLOR)
}

pub enum Direction {
    Left,
    Right,
    Down,
}

pub enum MoveResult {
    Moved,
    Collided,
    OutOfBounds,
    Spawned,
    NoTetromino,
}

#[derive(Debug)]
pub struct TetrisBoard {
    tetromino: Option<Tetromino>,
    pub cells: GameBoard,
}

#[derive(Debug)]
pub struct Tetromino {
    urcrnr_x: usize,
    urcrnr_y: usize,
    box_size: usize,
    pixels: Vec<(usize, usize)>,
    color: u8,
}

impl Tetromino {
    fn new(tetromino_type: TetrominoType, color: u8) -> Self {
        match tetromino_type {
            TetrominoType::I => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 4) / 2,
                urcrnr_y: 0,
                box_size: 4,
                pixels: vec![(0, 1), (1, 1), (2, 1), (3, 1)],
                color,
            },
            TetrominoType::J => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 3) / 2,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 0), (0, 1), (1, 1), (2, 1)],
                color,
            },
            TetrominoType::L => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 3) / 2,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 1), (1, 1), (2, 1), (2, 0)],
                color,
            },
            TetrominoType::O => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 2) / 2,
                urcrnr_y: 0,
                box_size: 2,
                pixels: vec![(0, 0), (1, 0), (1, 1), (0, 1)],
                color,
            },
            TetrominoType::S => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 3) / 2,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 1), (1, 1), (1, 0), (2, 0)],
                color,
            },
            TetrominoType::T => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 3) / 2,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 1), (1, 1), (1, 0), (2, 1)],
                color,
            },
            TetrominoType::Z => Self {
                urcrnr_x: (BOARD_WIDTH as usize - 3) / 2,
                urcrnr_y: 0,
                box_size: 3,
                pixels: vec![(0, 0), (1, 0), (1, 1), (2, 1)],
                color,
            },
        }
    }

    pub fn shift(&mut self, direction: Direction, board: &mut GameBoard) -> MoveResult {
        let (xo, yo) = match direction {
            Direction::Left => {
                if self.urcrnr_x == 0 {
                    return MoveResult::OutOfBounds;
                } else {
                    (self.urcrnr_x - 1, self.urcrnr_y)
                }
            }
            Direction::Right => (self.urcrnr_x + 1, self.urcrnr_y),
            Direction::Down => (self.urcrnr_x, self.urcrnr_y + 1),
        };
        for (px, py) in self.pixels.iter() {
            let ix = xo + *px;
            let iy = yo + *py;
            if ix >= BOARD_WIDTH as usize || iy >= BOARD_HEIGHT as usize {
                return MoveResult::OutOfBounds;
            }
            if board[iy][ix] > 0 {
                return MoveResult::Collided;
            }
        }
        self.urcrnr_x = xo;
        self.urcrnr_y = yo;
        MoveResult::Moved
    }

    pub fn rotate(&mut self, clockwise: bool, board: &mut GameBoard) -> MoveResult {
        let xo = (self.box_size / 2) as i8;
        let yo = (self.box_size / 2) as i8;
        let mut rotated_pixels = Vec::new();
        let mut shift_left = false;
        let mut shift_up = false;
        let par_size = self.box_size % 2 == 0;
        for (px, py) in self.pixels.iter() {
            let mut x = *px as i8;
            let mut y = *py as i8;
            if par_size && x >= xo as i8 {
                x += 1;
            }
            if par_size && y >= yo as i8 {
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
            shift_left = shift_left || (xr >= self.box_size as i8);
            shift_up = shift_up || (yr >= self.box_size as i8);
            rotated_pixels.push((xr as usize, yr as usize))
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
            if ix >= BOARD_WIDTH as usize || iy >= BOARD_HEIGHT as usize {
                return MoveResult::OutOfBounds;
            }
            if board[iy][ix] != 0 {
                return MoveResult::Collided;
            }
        }
        self.pixels = rotated_pixels;
        MoveResult::Moved
    }
}

impl TetrisBoard {
    pub fn new() -> Self {
        Self {
            tetromino: None,
            cells: Default::default(),
        }
    }

    pub fn spawn_tetromino(&mut self) -> MoveResult {
        let tetromino = Tetromino::new(TetrominoType::random(), get_random_color());
        for (px, py) in tetromino.pixels.iter() {
            let x = tetromino.urcrnr_x + *px;
            let y = tetromino.urcrnr_y + *py;
            if self.cells[y][x] != 0 {
                return MoveResult::Collided;
            }
        }
        self.tetromino = Some(tetromino);
        MoveResult::Spawned
    }

    pub fn rotate(&mut self) -> MoveResult {
        match &mut self.tetromino {
            Some(tetromino) => tetromino.rotate(true, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn move_left(&mut self) -> MoveResult {
        match &mut self.tetromino {
            Some(tetromino) => tetromino.shift(Direction::Left, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn move_right(&mut self) -> MoveResult {
        match &mut self.tetromino {
            Some(tetromino) => tetromino.shift(Direction::Right, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn move_down(&mut self) -> MoveResult {
        match &mut self.tetromino {
            Some(tetromino) => tetromino.shift(Direction::Down, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn get_tetromino_cells(&self) -> Option<(Vec<(usize, usize)>, u8)> {
        match &self.tetromino {
            Some(tetromino) => {
                let mut cells = Vec::new();
                for (px, py) in tetromino.pixels.iter() {
                    let x = tetromino.urcrnr_x + *px;
                    let y = tetromino.urcrnr_y + *py;
                    cells.push((x, y));
                }
                Some((cells, tetromino.color))
            }
            None => None,
        }
    }
}
