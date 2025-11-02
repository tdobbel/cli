pub const BOARD_WIDTH: u16 = 12;
pub const BOARD_HEIGHT: u16 = 24;

pub enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

pub struct TetrisBoard {
    nx: usize,
    ny: usize,
    cells: Vec<Vec<u8>>,
}

pub struct Tetromino {
    x: usize,
    y: usize,
    box_size: usize,
    cells: Vec<(usize, usize)>,
}

impl Tetromino {
    fn new(tetromino_type: TetrominoType) -> Self {
        match tetromino_type {
            TetrominoType::I => Self {
                x: 0,
                y: 0,
                box_size: 4,
                cells: vec![(0, 2), (1, 2), (2, 2), (3, 2)],
            },
            TetrominoType::J => Self {
                x: 0,
                y: 0,
                box_size: 3,
                cells: vec![(0, 2), (0, 1), (1, 1), (2, 1)],
            },
            TetrominoType::L => Self {
                x: 0,
                y: 0,
                box_size: 3,
                cells: vec![(0, 1), (1, 1), (2, 1), (2, 2)],
            },
            TetrominoType::O => Self {
                x: 0,
                y: 0,
                box_size: 2,
                cells: vec![(0, 0), (1, 0), (1, 1), (0, 1)],
            },
            TetrominoType::S => Self {
                x: 0,
                y: 0,
                box_size: 3,
                cells: vec![(0, 1), (1, 1), (1, 2), (2, 2)],
            },
            TetrominoType::T => Self {
                x: 0,
                y: 0,
                box_size: 3,
                cells: vec![(0, 1), (1, 1), (1, 2), (2, 1)],
            },
            TetrominoType::Z => Self {
                x: 0,
                y: 0,
                box_size: 3,
                cells: vec![(0, 2), (1, 2), (1, 1), (2, 1)],
            },
        }
    }
}

impl TetrisBoard {
    pub fn new(nx: usize, ny: usize) -> Self {
        let cells = vec![vec![0; nx]; ny];
        Self { nx, ny, cells }
    }
}
