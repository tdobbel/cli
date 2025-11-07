use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;

pub const BOARD_WIDTH: u16 = 10;
pub const BOARD_HEIGHT: u16 = 18;
pub const COLORS: [u8; 7] = [
    69,  // cornflowerblue
    122, // aquamarine
    141, // medium purple
    155, // darkolivegreen 2
    204, // indian red
    208, // darkorange
    226, // yellow 1
];

type GameBoard = [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

#[derive(Copy, Clone, Debug)]
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
    pub fn random_sequence() -> [Self; 7] {
        use TetrominoType::*;
        let mut types = [I, O, T, S, Z, J, L];
        let mut rng = rand::rng();
        types.shuffle(&mut rng);
        types
    }
}

fn random_color_sequence() -> [u8; 7] {
    let mut rng = rand::rng();
    let mut color_seq = COLORS;
    color_seq.shuffle(&mut rng);
    color_seq
}

pub enum Message {
    Drop,
    Pause,
    Unpause,
    Reset,
}

pub enum Direction {
    Left,
    Right,
    Down,
}

#[derive(Debug)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug)]
pub struct Tetromino {
    pub urcrnr_x: i16,
    pub urcrnr_y: i16,
    box_size: usize,
    pub pixels: [(i16, i16); 4],
    pub color: u8,
    pub mino_type: TetrominoType,
}

impl Tetromino {
    fn new(mino_type: TetrominoType, color: u8) -> Self {
        let xmax = BOARD_WIDTH as i16;
        let (box_size, pixels) = match mino_type {
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
            mino_type,
        }
    }

    pub fn shift(&mut self, direction: Direction, board: &mut GameBoard) -> bool {
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
                return false;
            }
            if board[iy as usize][ix as usize] > 0 {
                return false;
            }
        }
        self.urcrnr_x = xo;
        self.urcrnr_y = yo;
        true
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
        let even_size = self.box_size.is_multiple_of(2);
        for (i, (px, py)) in self.pixels.iter().enumerate() {
            let mut x = *px;
            let mut y = *py;
            if even_size && x >= xo {
                x += 1;
            }
            if even_size && y >= yo {
                y += 1;
            }
            let (mut xr, mut yr) = if clockwise {
                (xo + (y - yo), yo - (x - xo))
            } else {
                (xo - (y - yo), yo + (x - xo))
            };
            if even_size && xr >= xo {
                xr -= 1;
            }
            if even_size && yr >= yo {
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

#[derive(Debug)]
pub struct Game {
    pub current_mino: Option<Tetromino>,
    pub board: GameBoard,
    timer_tx: Sender<Message>,
    pub game_state: GameState,
    pub timer_rx: Receiver<Message>,
    pub timer_handle: JoinHandle<()>,
    next_minos: [TetrominoType; 7],
    next_colors: [u8; 7],
    inext: usize,
}

impl Game {
    pub fn new() -> Arc<Mutex<Self>> {
        let (timer_tx, timer_receiver) = mpsc::channel();
        let (timer_sender, timer_rx) = mpsc::channel();
        let timer_handle = thread::spawn(move || {
            game_timer(timer_receiver, timer_sender);
        });
        let next_minos = TetrominoType::random_sequence();
        let next_colors = random_color_sequence();
        let mut game = Self {
            current_mino: None,
            board: Default::default(),
            game_state: GameState::Playing,
            timer_tx,
            timer_rx,
            timer_handle,
            next_minos,
            next_colors,
            inext: 0,
        };
        game.first_draw();

        Arc::new(Mutex::new(game))
    }

    pub fn first_draw(&mut self) {
        loop {
            self.spawn_tetromino();
            let mino = self.current_mino.as_ref().unwrap();
            match mino.mino_type {
                TetrominoType::I | TetrominoType::L | TetrominoType::J | TetrominoType::T => break,
                _ => {}
            }
        }
    }

    pub fn kill_row(&mut self, row_indx: usize) {
        self.board[row_indx].iter_mut().for_each(|c| *c = 0);
        for y in (1..=row_indx).rev() {
            for x in 0..BOARD_WIDTH as usize {
                self.board[y][x] = self.board[y - 1][x]
            }
        }
        self.board[0].iter_mut().for_each(|c| *c = 0);
    }

    pub fn check_rows(&mut self) {
        for i in (1..BOARD_HEIGHT as usize).rev() {
            let nblock = self.board[i].iter().filter(|&c| *c > 0).count();
            if nblock == BOARD_WIDTH as usize {
                self.kill_row(i);
            }
        }
    }

    pub fn update(&mut self) {
        self.check_rows();

        let mut drop_count = 0;
        while !&self.timer_rx.try_recv().is_err() {
            drop_count += 1;
        }
        (0..drop_count).for_each(|_| self.move_down());
    }

    pub fn game_over(&mut self) {
        self.game_state = GameState::GameOver;
        self.timer_tx.send(Message::Reset).unwrap();
        self.timer_tx.send(Message::Pause).unwrap();
        self.board = Default::default();
    }

    pub fn reset(&mut self) {
        self.game_state = GameState::Playing;
        self.board = Default::default();
        self.timer_tx.send(Message::Reset).unwrap();
        self.first_draw();
    }

    pub fn spawn_tetromino(&mut self) {
        if self.inext == 7 {
            self.next_minos = TetrominoType::random_sequence();
            self.next_colors = random_color_sequence();
            self.inext = 0;
        }
        let tetromino = Tetromino::new(self.next_minos[self.inext], self.next_colors[self.inext]);
        self.inext += 1;
        for (px, py) in tetromino.pixels.iter() {
            let x = tetromino.urcrnr_x + *px;
            let y = tetromino.urcrnr_y + *py;
            if self.board[y as usize][x as usize] != 0 {
                self.game_over();
            }
        }
        self.current_mino = Some(tetromino);
    }

    pub fn new_tetromino(&mut self) {
        let mino = self.current_mino.as_ref().unwrap();
        for px in mino.pixels.iter() {
            let x = (mino.urcrnr_x + px.0) as usize;
            let y = (mino.urcrnr_y + px.1) as usize;
            self.board[y][x] = mino.color;
        }
        self.spawn_tetromino();
    }

    pub fn rotate(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
        if let Some(mino) = &mut self.current_mino {
            mino.rotate(true, &mut self.board);
        }
    }

    pub fn move_left(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
        if let Some(mino) = &mut self.current_mino {
            mino.shift(Direction::Left, &mut self.board);
        }
    }

    pub fn move_right(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
        if let Some(mino) = &mut self.current_mino {
            mino.shift(Direction::Right, &mut self.board);
        }
    }

    pub fn move_down(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
        if let Some(mino) = &mut self.current_mino
            && !mino.shift(Direction::Down, &mut self.board)
        {
            self.new_tetromino();
        }
    }

    pub fn hard_drop(&mut self) {
        if !matches!(self.game_state, GameState::Playing) {
            return;
        }
        if let Some(mino) = &mut self.current_mino {
            while mino.shift(Direction::Down, &mut self.board) {}
            self.new_tetromino();
        }
    }

    pub fn toggle_paused(&mut self) {
        match self.game_state {
            GameState::Playing => {
                self.game_state = GameState::Paused;
                let _ = self.timer_tx.send(Message::Pause);
            }
            GameState::Paused => {
                self.game_state = GameState::Playing;
                let _ = self.timer_tx.send(Message::Unpause);
            }
            _ => {}
        }
    }
}

struct Timer {
    level: u8,
    duration: u128,
}

impl Timer {
    fn new(start: u8) -> Self {
        Self {
            level: start,
            duration: 48 * 17,
        }
    }

    // fn increase(&mut self) {
    //     self.level += 1;
    //     self.duration = get_drop_time_duration(self.level);
    // }
}

fn game_timer(timer_receiver: Receiver<Message>, timer_sender: Sender<Message>) {
    let mut time = Instant::now();
    let mut timer = Timer::new(0);

    loop {
        thread::sleep(Duration::from_millis(16));
        let elapsed = time.elapsed().as_millis();
        if elapsed >= timer.duration {
            if let Ok(signal) = timer_receiver.try_recv() {
                match signal {
                    // SIGNAL_INCREASE => timer.increase(),
                    Message::Pause => {
                        loop {
                            thread::sleep(Duration::from_millis(250)); //recheck every quarter second
                            if let Ok(signal) = timer_receiver.try_recv() {
                                match signal {
                                    Message::Unpause => break,
                                    Message::Reset => {
                                        timer = Timer::new(0);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Message::Reset => timer = Timer::new(0),
                    _ => {}
                }
            }
            time = Instant::now();
            timer_sender.send(Message::Drop).unwrap();
        }
    }
}
