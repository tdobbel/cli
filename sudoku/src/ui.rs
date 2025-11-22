use crate::big_text::*;
use crate::game::{Game, GameState};

const NUMBERS: [&str; 9] = [ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE];

use std::{
    io,
    sync::{Arc, Mutex, mpsc::Receiver},
};

use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Padding, Paragraph, Widget},
};

fn draw_board(game: &Game, cell_areas: &[Vec<Rect>], buf: &mut Buffer) {
    for (i, row) in cell_areas.iter().enumerate() {
        for (j, rect) in row.iter().enumerate() {
            let case_x = j / 3;
            let case_y = i / 3;
            let block = Block::default().padding(Padding::new(3, 0, 1, 0));
            if (case_x + case_y) % 2 == 0 {
                Block::default().bg(Color::Indexed(17)).render(*rect, buf);
            }
            let color = if game.board_state[i][j] == 2 {
                Color::Indexed(230)
            } else {
                match game.game_state {
                    GameState::Done => {
                        if game.board_state[i][j] == 1 {
                            Color::Green
                        } else {
                            Color::Red
                        }
                    }
                    _ => Color::White,
                }
            };
            if game.sudoku.grid[i][j] > 0 {
                let num_indx = (game.sudoku.grid[i][j] - 1) as usize;
                Paragraph::new(NUMBERS[num_indx])
                    .style(Style::default().fg(color))
                    .block(block)
                    .render(*rect, buf);
            } else if let Some(notes) = game.notes.get(&(i, j)) {
                let note_text = notes
                    .iter()
                    .map(|&n| n.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");
                Paragraph::new(note_text).block(block).render(*rect, buf);
            }
            if (i, j) == game.current_pos {
                let color = match game.game_state {
                    GameState::Normal => Color::White,
                    GameState::Note => Color::Yellow,
                    GameState::Done => Color::Red,
                };
                Block::bordered().fg(color).render(*rect, buf);
            }
        }
    }
}

fn create_layout(area: &Rect) -> Vec<Vec<Rect>> {
    let [outer] = Layout::horizontal([Constraint::Length(100)])
        .flex(Flex::Center)
        .areas(*area);
    let [grid_area] = Layout::vertical([Constraint::Length(63)])
        .flex(Flex::Center)
        .areas(outer);

    let col_constraints = (0..9).map(|_| Constraint::Length(10));
    let row_constraints = (0..9).map(|_| Constraint::Length(7));
    let horizontal = Layout::horizontal(col_constraints);
    let vertical = Layout::vertical(row_constraints);
    let rows = vertical.split(grid_area);
    let mut areas = Vec::new();
    for row in rows.iter() {
        let cols = horizontal.split(*row);
        areas.push(cols.to_vec());
    }
    areas
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grids = create_layout(&area);
        draw_board(self, &grids, buf);
    }
}

pub fn draw_ui(
    mut terminal: DefaultTerminal,
    game_state: Arc<Mutex<Game>>,
    stop_receiver: Receiver<()>,
) -> io::Result<()> {
    loop {
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
