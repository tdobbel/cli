use crate::big_text::*;
use crate::game::{
    BLUE, BOARD_HEIGHT, BOARD_WIDTH, GREEN, Game, GameState, ORANGE, PURPLE, RED, Tetromino, YELLOW,
};

use std::{
    io,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
    time::Duration,
};

use ratatui::widgets::Padding;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

const UNIT_X: u16 = 2;
const UNIT_Y: u16 = 1;

pub fn message_area(area: Rect, width: u16, height: u16) -> Rect {
    let [message_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [message_area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(message_area);
    message_area
}

fn draw_big_text(text: &str, area: &Rect, buf: &mut Buffer, style: &Style) {
    let height = text.lines().count() as u16;
    let msg_area = message_area(*area, BIG_TEXT_WIDTH, height + 2);
    let block = Block::bordered().bg(Color::Black);
    Paragraph::new(text)
        .block(block)
        .style(*style)
        .render(msg_area, buf);
}

fn draw_board(game: &Game, board_area: &Rect, buf: &mut Buffer) {
    Block::bordered()
        .border_set(border::THICK)
        .render(*board_area, buf);
    let inner_area = Rect::new(
        board_area.x + UNIT_X,
        board_area.y + UNIT_Y,
        UNIT_X * BOARD_WIDTH,
        UNIT_X * BOARD_HEIGHT,
    );
    let col_constraints = (0..BOARD_WIDTH as usize).map(|_| Constraint::Length(UNIT_X));
    let row_constraints = (0..BOARD_HEIGHT as usize).map(|_| Constraint::Length(UNIT_Y));
    let horizontal = Layout::horizontal(col_constraints).spacing(0);
    let vertical = Layout::vertical(row_constraints).spacing(0);
    let rows = vertical.split(inner_area);
    for (y, row) in rows.iter().enumerate() {
        let cols = horizontal.split(*row);
        for (x, cell) in cols.iter().enumerate() {
            let color = Color::Indexed(game.board[y][x]);
            Block::default().bg(color).render(*cell, buf);
        }
        if let Some(mino) = &game.current_mino {
            for (pos_x, pos_y) in mino.pixels.iter() {
                let x = (mino.urcrnr_x + pos_x) as usize;
                let y = (mino.urcrnr_y + pos_y) as usize;
                let cell = horizontal.split(rows[y])[x];
                Block::default()
                    .bg(Color::Indexed(mino.color))
                    .render(cell, buf);
            }
        }
    }
}

fn draw_title(area: &Rect, buf: &mut Buffer) {
    let letters = [
        BIG_TETRIS_T,
        BIG_TETRIS_E,
        BIG_TETRIS_T,
        BIG_TETRIS_R,
        BIG_TETRIS_I,
        BIG_TETRIS_S,
    ];
    let colors: [u8; 6] = [RED, ORANGE, YELLOW, GREEN, BLUE, PURPLE];
    let widths: [u16; 6] = [4, 4, 4, 5, 2, 5];
    let constraints = widths.iter().map(|&w| Constraint::Length(w));
    let layouts = Layout::horizontal(constraints).flex(Flex::SpaceBetween);
    let letter_areas = layouts.split(*area);
    for (i, letter_area) in letter_areas.iter().enumerate() {
        let style = Style::new().fg(Color::Indexed(colors[i]));
        Paragraph::new(letters[i])
            .style(style)
            .render(*letter_area, buf);
    }
}

fn create_layout(area: &Rect) -> [Rect; 4] {
    let board_width = UNIT_X * (BOARD_WIDTH + 2);
    let board_height = UNIT_Y * (BOARD_HEIGHT + 2);
    let stats_width = UNIT_X * STAT_WIDTH;
    let [outer] = Layout::horizontal([Constraint::Length(board_width + stats_width)])
        .flex(Flex::Center)
        .areas(*area);

    let [top_pane, bottom_pane] = Layout::vertical([
        Constraint::Length(TITLE_HEIGHT),
        Constraint::Length(board_height),
    ])
    .flex(Flex::Center)
    .areas(outer);
    let [title_area] = Layout::horizontal([Constraint::Length(TITLE_WIDTH)])
        .flex(Flex::Center)
        .areas(top_pane);
    let [board_area, right_pane] = Layout::horizontal([
        Constraint::Length(board_width),
        Constraint::Length(stats_width),
    ])
    .areas(bottom_pane);
    let stats_height = STAT_HEIGHT * UNIT_Y;
    let next_area = Rect::new(right_pane.x, right_pane.y, stats_width, 7);
    let stats_area = Rect::new(
        right_pane.x,
        right_pane.y + right_pane.height - stats_height,
        stats_width,
        stats_height,
    );
    [title_area, board_area, next_area, stats_area]
}

fn draw_statistics(game: &Game, area: &Rect, buf: &mut Buffer) {
    let block = Block::bordered().padding(Padding::new(1, 1, 0, 0));
    let stats_text = format!(
        "LEVEL\n      {:02}\n\nLINES\n     {:03}\n\nSCORE\n  {:06}\n\n",
        game.level, game.line_count, game.score
    );
    Paragraph::new(stats_text).block(block).render(*area, buf);
}

pub fn draw_next(mino: Tetromino, area: &Rect, buf: &mut Buffer) {
    let block = Block::bordered().padding(Padding::new(1, 1, 0, 0));
    Paragraph::new("NEXT").block(block).render(*area, buf);
    let mino_size = mino.box_size;
    let mino_rect = Rect::new(
        area.x + UNIT_X,
        area.y + 3 * UNIT_Y,
        mino_size as u16 * UNIT_X,
        mino_size as u16 * UNIT_Y,
    );
    let col_constraints = (0..mino_size).map(|_| Constraint::Length(UNIT_X));
    let row_constraints = (0..mino_size).map(|_| Constraint::Length(UNIT_Y));
    let horizontal = Layout::horizontal(col_constraints).spacing(0);
    let vertical = Layout::vertical(row_constraints).spacing(0);
    let color = Color::Indexed(mino.color);
    for (x, y) in mino.pixels.iter() {
        let row = vertical.split(mino_rect)[*y as usize];
        let cell = horizontal.split(row)[*x as usize];
        Block::default().bg(color).render(cell, buf);
    }
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title_area, board_area, next_area, stats_area] = create_layout(&area);
        draw_title(&title_area, buf);
        draw_board(self, &board_area, buf);
        draw_statistics(self, &stats_area, buf);
        draw_next(self.get_next_mino(), &next_area, buf);
        match self.game_state {
            GameState::Playing => {}
            GameState::Paused => {
                let style = Style::new().yellow().bold();
                draw_big_text(BIG_TEXT_PAUSED, &area, buf, &style);
            }
            GameState::GameOver => {
                let style = Style::new().red().bold();
                draw_big_text(GAME_OVER_TEXT, &area, buf, &style);
            }
        }
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
