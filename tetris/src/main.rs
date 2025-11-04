use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Padding, Paragraph, Widget},
};
mod tetris;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = TetrisApp::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

const UNIT_X: u16 = 2;
const UNIT_Y: u16 = 1;

#[derive(Debug)]
enum GameState {
    Running,
    Paused,
    GameOver,
    Quitting,
}

impl GameState {
    fn is_quitting(&self) -> bool {
        matches!(self, GameState::Quitting)
    }
}

#[derive(Debug)]
pub struct TetrisApp {
    state: GameState,
    game: tetris::TetrisBoard,
}

impl TetrisApp {
    fn new() -> Self {
        let mut game = tetris::TetrisBoard::new();
        game.spawn_tetromino();
        Self {
            state: GameState::Paused,
            game,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.state.is_quitting() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    // ANCHOR: handle_key_event fn
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.state = GameState::Quitting,
            KeyCode::Up => {
                self.game.rotate();
            }
            KeyCode::Right => {
                self.game.move_right();
            }
            KeyCode::Left => {
                self.game.move_left();
            }
            _ => {}
        }
    }
}
impl Widget for &TetrisApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = UNIT_X * tetris::BOARD_WIDTH;
        let height = UNIT_Y * tetris::BOARD_HEIGHT;
        let [game_border_area] = Layout::horizontal([Constraint::Length(width + 6)])
            .flex(Flex::Center)
            .areas(area);
        let [game_border_area] = Layout::vertical([Constraint::Length(height + 3)])
            .flex(Flex::Center)
            .areas(game_border_area);
        let [game_area] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(area);
        let [game_area] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Center)
            .areas(game_area);
        let title = Line::from(" Tetris Game ".bold());
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
            " Start/Pause".into(),
            "<Space> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        block.render(game_border_area, buf);
        let col_constraints = (0..tetris::BOARD_WIDTH as usize).map(|_| Constraint::Length(UNIT_X));
        let row_constraints =
            (0..tetris::BOARD_HEIGHT as usize).map(|_| Constraint::Length(UNIT_Y));
        let horizontal = Layout::horizontal(col_constraints).spacing(0);
        let vertical = Layout::vertical(row_constraints).spacing(0);

        let rows = vertical.split(game_area);
        for (y, row) in rows.iter().enumerate() {
            let cols = horizontal.split(*row);
            for (x, cell) in cols.iter().enumerate() {
                let color = Color::Indexed(self.game.cells[y][x]);
                Block::default().bg(color).render(*cell, buf);
            }
            if let Some((pixels, color)) = self.game.get_tetromino_cells() {
                for (px, py) in pixels.iter() {
                    let cols = horizontal.split(rows[*py]);
                    let cell = &cols[*px];
                    Block::default()
                        .bg(Color::Indexed(color))
                        .render(*cell, buf);
                }
            }
        }
    }
}
