use crate::game::{EMPTY, Game, GameState, MINE, STATE_FLAGGED, STATE_HIDDEN};

use std::{
    io,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
    time::Duration,
};

use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub const NUMBER_COLORS: [u8; 8] = [51, 46, 160, 27, 88, 50, 220, 189];

fn draw_selection_menu(game: &Game, menu_area: &Rect, buf: &mut Buffer) {
    if matches!(game.game_state, GameState::Playing) {
        return;
    }
    let title_area = Rect::new(menu_area.x, menu_area.y, menu_area.width, 3);
    let text = match game.game_state {
        GameState::ChangeLevel => Paragraph::new("PAUSE").style(Style::new().yellow().bold()),
        GameState::GameOver => Paragraph::new("GAME OVER!").style(Style::new().red().bold()),
        GameState::Victory => Paragraph::new("VICTORY!").style(Style::new().green().bold()),
        GameState::Playing => panic!("This should not be called in Playing state"),
    };
    text.block(Block::bordered().fg(Color::White))
        .alignment(Alignment::Center)
        .render(title_area, buf);
    let menu_text = vec![
        "SELECT LEVEL:".into(),
        "".into(),
        Line::from(vec![
            "1".green().bold(),
            " - ".into(),
            "Beginner".bold(),
            "       (9x9, 10 mines)".into(),
        ]),
        Line::from(vec![
            "2".blue().bold(),
            " - ".into(),
            "Intermediate".bold(),
            " (16x16, 40 mines)".into(),
        ]),
        Line::from(vec![
            "3".red().bold(),
            " - ".into(),
            "Expert".bold(),
            "       (30x16, 99 mines)".into(),
        ]),
    ];
    Paragraph::new(menu_text)
        .block(Block::bordered().fg(Color::White))
        .alignment(Alignment::Center)
        .render(
            Rect::new(
                menu_area.x,
                menu_area.y + 3,
                menu_area.width,
                menu_area.height - 3,
            ),
            buf,
        );
}

fn intersect(rect1: &Rect, rect2: &Rect) -> bool {
    let x_overlap = rect1.x < rect2.x + rect2.width && rect2.x < rect1.x + rect1.width;
    let y_overlap = rect1.y < rect2.y + rect2.height && rect2.y < rect1.y + rect1.height;
    x_overlap && y_overlap
}

fn draw_board(game: &Game, board_area: &Rect, menu_area: &Rect, buf: &mut Buffer) {
    let col_constraints = (0..game.nx as usize).map(|_| Constraint::Length(1));
    let row_constraints = (0..game.ny as usize).map(|_| Constraint::Length(1));
    let status_bar = Rect::new(board_area.x, board_area.y, board_area.width, 2);
    let logo = match game.game_state {
        GameState::GameOver => "☢".red(),
        _ => "▶ ".red(),
    };
    Paragraph::new(Line::from(vec![
        logo,
        format!(" {}/{}", game.n_flagged, game.n_mines).into(),
    ]))
    .alignment(Alignment::Center)
    .render(status_bar, buf);
    let mine_field = Rect::new(
        board_area.x,
        board_area.y + 1,
        board_area.width,
        board_area.height - 1,
    );
    let show_menu = !matches!(game.game_state, GameState::Playing);
    let horizontal = Layout::horizontal(col_constraints).spacing(1);
    let vertical = Layout::vertical(row_constraints).spacing(0);
    let rows = vertical.split(mine_field);
    for (y, row) in rows.iter().enumerate() {
        let cols = horizontal.split(*row);
        for (x, cell) in cols.iter().enumerate() {
            if show_menu && intersect(cell, menu_area) {
                continue;
            }
            if x == game.current_x as usize && y == game.current_y as usize {
                let rect = Rect::new(cell.x - 1, cell.y, cell.width + 2, cell.height);
                Block::default().bg(Color::Indexed(240)).render(rect, buf);
            }
            let text = match game.state[y][x] {
                STATE_HIDDEN => Paragraph::new("■"),
                STATE_FLAGGED => Paragraph::new("▶").style(Style::default().fg(Color::Red)),
                _ => {
                    if game.board[y][x] == MINE {
                        Paragraph::new("☢")
                    } else if game.board[y][x] == EMPTY {
                        Paragraph::new("·")
                    } else {
                        let cntr = game.board[y][x] as usize;
                        let color = Color::Indexed(NUMBER_COLORS[cntr - 1]);
                        Paragraph::new(format!("{cntr}"))
                            .style(Style::default().fg(color))
                            .bold()
                    }
                }
            };
            text.render(*cell, buf);
        }
    }
    draw_selection_menu(game, menu_area, buf);
}

fn create_layout(game: &Game, area: &Rect) -> [Rect; 2] {
    let [outer] = Layout::horizontal([Constraint::Length(2 * game.nx - 1)])
        .flex(Flex::Center)
        .areas(*area);
    let [board_area] = Layout::vertical([Constraint::Length(game.ny + 1)])
        .flex(Flex::Center)
        .areas(outer);
    let [outer] = Layout::horizontal([Constraint::Length(38)])
        .flex(Flex::Center)
        .areas(*area);
    let [menu_area] = Layout::vertical([Constraint::Length(11)])
        .flex(Flex::Center)
        .areas(outer);
    [board_area, menu_area]
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [board_area, menu_area] = create_layout(self, &area);
        draw_board(self, &board_area, &menu_area, buf);
        let rect = Rect::new(
            board_area.x,
            board_area.y + board_area.height + 1,
            board_area.width,
            4,
        );
        Paragraph::new(vec![
            Line::from(vec!["<Space>".blue().bold(), " Reveal".into()]),
            Line::from(vec!["<f>".blue().bold(), " Toggle flag".into()]),
            Line::from(vec!["<Esc>".blue().bold(), " Start/Pause".into()]),
            Line::from(vec!["<q>".blue().bold(), " Quit".into()]),
        ])
        .render(rect, buf);

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
