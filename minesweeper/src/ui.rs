use crate::game::{EMPTY, Game, MINE, STATE_FLAGGED, STATE_HIDDEN};

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
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    widgets::{Block, BorderType, Paragraph, Widget},
};

fn draw_board(game: &Game, board_area: &Rect, buf: &mut Buffer) {
    let col_constraints = (0..game.nx as usize).map(|_| Constraint::Length(1));
    let row_constraints = (0..game.ny as usize).map(|_| Constraint::Length(1));
    let horizontal = Layout::horizontal(col_constraints).spacing(1);
    let vertical = Layout::vertical(row_constraints).spacing(0);
    let rows = vertical.split(*board_area);
    for (y, row) in rows.iter().enumerate() {
        let cols = horizontal.split(*row);
        for (x, cell) in cols.iter().enumerate() {
            if x == game.current_x as usize && y == game.current_y as usize {
                let rect = Rect::new(cell.x - 1, cell.y, cell.width + 2, cell.height);
                Block::default().bg(Color::Indexed(240)).render(rect, buf);
            }
            let text = match game.state[y][x] {
                STATE_HIDDEN => Paragraph::new("■"),
                STATE_FLAGGED => Paragraph::new("▶"),
                _ => {
                    if game.board[y][x] == MINE {
                        Paragraph::new("☠")
                    } else if game.board[y][x] == EMPTY {
                        Paragraph::new("·")
                    } else {
                        Paragraph::new("1")
                    }
                }
            };
            text.render(*cell, buf);
        }
    }
}

fn create_layout(game: &Game, area: &Rect) -> Rect {
    let [outer] = Layout::horizontal([Constraint::Length(2 * game.nx - 1)])
        .flex(Flex::Center)
        .areas(*area);

    let [board_area] = Layout::vertical([Constraint::Length(game.ny)])
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
