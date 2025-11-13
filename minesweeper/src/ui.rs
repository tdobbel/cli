use crate::game::{Game, STATE_HIDDEN};

use std::{
    io,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
    time::Duration,
};

// use ratatui::widgets::Padding;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    widgets::{Block, BorderType, Paragraph, Widget},
};

pub const UNIT_X: u16 = 4;
pub const UNIT_Y: u16 = 2;
pub const HIDDEN_COLOR: u8 = 241;
pub const REVEALED_COLOR: u8 = 250;

fn draw_board(game: &Game, board_area: &Rect, buf: &mut Buffer) {
    let col_constraints = (0..game.nx as usize).map(|_| Constraint::Length(UNIT_X));
    let row_constraints = (0..game.ny as usize).map(|_| Constraint::Length(UNIT_Y));
    let horizontal = Layout::horizontal(col_constraints).spacing(1);
    let vertical = Layout::vertical(row_constraints).spacing(1);
    let rows = vertical.split(*board_area);
    for (y, row) in rows.iter().enumerate() {
        let cols = horizontal.split(*row);
        for (x, cell) in cols.iter().enumerate() {
            if game.state[y][x] == STATE_HIDDEN {
                Block::default()
                    .bg(Color::Indexed(HIDDEN_COLOR))
                    .render(*cell, buf);
            }
            if x == game.current_x as usize && y == game.current_y as usize {
                Block::bordered().fg(Color::Yellow).render(*cell, buf);
            }
        }
    }
}

fn create_layout(game: &Game, area: &Rect) -> Rect {
    let board_width = game.nx * UNIT_X;
    let board_height = game.ny * UNIT_Y;
    let [outer] = Layout::horizontal([Constraint::Length(board_width + game.nx)])
        .flex(Flex::Center)
        .areas(*area);

    let [board_area] = Layout::vertical([Constraint::Length(board_height + game.ny)])
        .flex(Flex::Center)
        .areas(outer);
    board_area
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let board_area = create_layout(self, &area);
        draw_board(self, &board_area, buf);
        // match self.game_state {
        //     GameState::Playing => {}
        //     GameState::Paused => {
        //         let style = Style::new().yellow().bold();
        //         draw_big_text(BIG_TEXT_PAUSED, &area, buf, &style);
        //     }
        //     GameState::GameOver => {
        //         let style = Style::new().red().bold();
        //         draw_big_text(GAME_OVER_TEXT, &area, buf, &style);
        //     }
        // }
    }
}

pub fn draw_ui(
    mut terminal: DefaultTerminal,
    game_state: Arc<Mutex<Game>>,
    stop_receiver: Receiver<()>,
) -> io::Result<()> {
    loop {
        thread::sleep(Duration::from_millis(16));
        match stop_receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            _ => break,
        }

        terminal
            .draw(|frame| {
                let mut game = game_state.lock().unwrap();

                game.update();

                frame.render_widget(&*game, frame.area());
            })
            .map(|_| ())?;
    }
    Ok(())
}
