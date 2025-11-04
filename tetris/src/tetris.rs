use rand::Rng;

pub const BOARD_WIDTH: u16 = 12;
pub const BOARD_HEIGHT: u16 = 24;
pub const COLORS: [u8; 8] = [2, 9, 11, 12, 155, 178, 199, 208];

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
    COLORS[rng.random_range(0..COLORS.len())]
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
    current_mino: Option<Tetromino>,
    pub cells: GameBoard,
}

#[derive(Debug)]
pub struct Tetromino {
    urcrnr_x: i16,
    urcrnr_y: i16,
    box_size: usize,
    pixels: [(i16, i16); 4],
    color: u8,
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
        Self {
            urcrnr_x: (xmax - box_size as i16) / 2,
            urcrnr_y: 0,
            box_size,
            pixels,
            color,
        }
    }

    pub fn shift(&mut self, direction: Direction, board: &mut GameBoard) -> MoveResult {
        let (xo, yo) = match direction {
            Direction::Left => (self.urcrnr_x - 1, self.urcrnr_y),
            Direction::Right => (self.urcrnr_x + 1, self.urcrnr_y),
            Direction::Down => (self.urcrnr_x, self.urcrnr_y + 1),
        };
        let xmax = BOARD_WIDTH as i16;
        let ymax = BOARD_HEIGHT as i16;
        for (px, py) in self.pixels.iter() {
            let ix = xo + *px;
            let iy = yo + *py;
            if ix < 0 || iy < 0 || ix >= xmax || iy >= ymax {
                return MoveResult::OutOfBounds;
            }
            if board[iy as usize][ix as usize] > 0 {
                return MoveResult::Collided;
            }
        }
        self.urcrnr_x = xo;
        self.urcrnr_y = yo;
        MoveResult::Moved
    }

    pub fn rotate(&mut self, clockwise: bool, board: &mut GameBoard) -> MoveResult {
        let size = self.box_size as i16;
        let xo = size / 2;
        let yo = size / 2;
        let mut rotated_pixels: [(i16, i16); 4] = Default::default();
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
            shift_left = shift_left || (xr >= size);
            shift_up = shift_up || (yr >= size);
            rotated_pixels[i] = (xr, yr)
        }
        if shift_up {
            rotated_pixels.iter_mut().for_each(|xy| xy.1 -= 1);
        }
        if shift_left {
            rotated_pixels.iter_mut().for_each(|xy| xy.0 -= 1);
        }
        let xmax = BOARD_WIDTH as i16;
        let ymax = BOARD_HEIGHT as i16;
        for (x, y) in rotated_pixels.iter() {
            let ix = self.urcrnr_x + *x;
            let iy = self.urcrnr_y + *y;
            if ix < 0 || iy < 0 || ix >= xmax || iy >= ymax {
                return MoveResult::OutOfBounds;
            }
            if board[iy as usize][ix as usize] != 0 {
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
            current_mino: None,
            cells: Default::default(),
        }
    }

    pub fn spawn_tetromino(&mut self) -> MoveResult {
        let tetromino = Tetromino::new(TetrominoType::random(), get_random_color());
        for (px, py) in tetromino.pixels.iter() {
            let x = tetromino.urcrnr_x + *px;
            let y = tetromino.urcrnr_y + *py;
            if self.cells[y as usize][x as usize] != 0 {
                return MoveResult::Collided;
            }
        }
        self.current_mino = Some(tetromino);
        MoveResult::Spawned
    }

    pub fn rotate(&mut self) -> MoveResult {
        match &mut self.current_mino {
            Some(tetromino) => tetromino.rotate(true, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn move_left(&mut self) -> MoveResult {
        match &mut self.current_mino {
            Some(tetromino) => tetromino.shift(Direction::Left, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn move_right(&mut self) -> MoveResult {
        match &mut self.current_mino {
            Some(tetromino) => tetromino.shift(Direction::Right, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn move_down(&mut self) -> MoveResult {
        match &mut self.current_mino {
            Some(tetromino) => tetromino.shift(Direction::Down, &mut self.cells),
            None => MoveResult::NoTetromino,
        }
    }

    pub fn get_tetromino_cells(&self) -> Option<([(usize, usize); 4], u8)> {
        match &self.current_mino {
            Some(mino) => {
                let mut cells: [(usize, usize); 4] = Default::default();
                for (i, pix) in mino.pixels.iter().enumerate() {
                    cells[i].0 = (mino.urcrnr_x + pix.0) as usize;
                    cells[i].1 = (mino.urcrnr_y + pix.1) as usize;
                }
                Some((cells, mino.color))
            }
            None => None,
        }
    }
}
