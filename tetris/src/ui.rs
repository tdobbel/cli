use crate::game::{BOARD_HEIGHT, BOARD_WIDTH, Game, GameState};

use std::{
    io,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
    time::Duration,
};

use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

const UNIT_X: u16 = 2;
const UNIT_Y: u16 = 1;

pub const BIG_TETRIS: &str = r#"█████ ████ ████ ████  ██  ███ 
  ██  ██    ██  ██ ██ ██ █   
  ██  ███   ██  ████  ██  ███ 
  ██  ██    ██  ██ ██ ██     █
  ██  ████  ██  ██ ██ ██  ███ 
"#;

pub const BIG_TEXT_PAUSED: &str = r#" ██████  ██████  ██  ██    ████    ██████  ████   
 ██  ██  ██  ██  ██  ██  ██        ██      ██  ██ 
 ██████  ██████  ██  ██    ████    ████    ██  ██ 
 ██      ██  ██  ██  ██        ██  ██      ██  ██ 
 ██      ██  ██  ██████    ████    ██████  ████   
"#;

pub const GAME_OVER_TEXT: &str = r#"   ████      ██      ██  ██    ██████             
 ██        ██  ██  ██  ██  ██  ██                 
 ██  ████  ██████  ██      ██  ████               
 ██    ██  ██  ██  ██      ██  ██                 
   ████    ██  ██  ██      ██  ██████             
                                                  
               ██    ██  ██  ██████  ██████    ██ 
             ██  ██  ██  ██  ██      ██    ██  ██ 
             ██  ██  ██  ██  ████    ██    ██  ██ 
             ██  ██  ██  ██  ██      ██████       
               ██      ██    ██████  ██    ██  ██ 
"#;

pub fn message_area(area: Rect, width: u16, height: u16) -> Rect {
    let [message_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [message_area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(message_area);
    message_area
}

fn draw_text(text: &str, area: &Rect, buf: &mut Buffer, style: &Style) {
    let height = text.lines().count() as u16;
    let width = text.lines().map(|line| line.len()).max().unwrap_or(0) as u16;
    let msg_area = message_area(*area, width / 2, height);
    Paragraph::new(text).style(*style).render(msg_area, buf);
}

fn draw_board(game: &Game, area: &Rect, buf: &mut Buffer, width: u16, height: u16) {
    let [game_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(*area);
    let [game_area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(game_area);
    let col_constraints = (0..BOARD_WIDTH as usize).map(|_| Constraint::Length(UNIT_X));
    let row_constraints = (0..BOARD_HEIGHT as usize).map(|_| Constraint::Length(UNIT_Y));
    let horizontal = Layout::horizontal(col_constraints).spacing(0);
    let vertical = Layout::vertical(row_constraints).spacing(0);
    let rows = vertical.split(game_area);
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

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = UNIT_X * BOARD_WIDTH;
        let height = UNIT_Y * BOARD_HEIGHT;
        let [game_border_area] = Layout::horizontal([Constraint::Length(width + 6)])
            .flex(Flex::Center)
            .areas(area);
        let [game_border_area] = Layout::vertical([Constraint::Length(height + 4)])
            .flex(Flex::Center)
            .areas(game_border_area);
        let title = Line::from(" Tetris Game ".bold());
        let instructions = Line::from(vec![
            " Quit".into(),
            "<q> ".blue().bold(),
            "Start/Pause".into(),
            "<Esc> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        block.render(game_border_area, buf);
        draw_board(self, &area, buf, width, height);
        match self.game_state {
            GameState::Playing => {}
            GameState::Paused => {
                let style = Style::new().yellow().bold();
                draw_text(BIG_TEXT_PAUSED, &area, buf, &style);
            }
            GameState::GameOver => {
                let style = Style::new().red().bold();
                draw_text(GAME_OVER_TEXT, &area, buf, &style);
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
