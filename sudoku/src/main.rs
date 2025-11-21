mod big_text;
mod game;
mod sudoku;
mod ui;

use std::{
    io::{self},
    thread,
};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

use ui::draw_ui;

use game::Game;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(terminal: DefaultTerminal) -> io::Result<()> {
    let (stop_sender, stop_receiver) = std::sync::mpsc::channel();

    let game = Game::new();
    let game_clone = game.clone();

    let draw_thread_handle = thread::spawn(|| -> io::Result<()> {
        draw_ui(terminal, game_clone, stop_receiver)?;
        Ok(())
    });

    loop {
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            let mut game = game.lock().unwrap();
            match key.code {
                KeyCode::Down => game.move_down(),
                KeyCode::Left => game.move_left(),
                KeyCode::Right => game.move_right(),
                KeyCode::Up => game.move_up(),
                // KeyCode::Esc => match game.game_state {
                //     GameState::Playing | GameState::ChangeLevel => game.toggle_level_selection(),
                //     _ => {}
                // },
                KeyCode::Char('1') => game.set_number(1),
                KeyCode::Char('2') => game.set_number(2),
                KeyCode::Char('3') => game.set_number(3),
                KeyCode::Char('4') => game.set_number(4),
                KeyCode::Char('5') => game.set_number(5),
                KeyCode::Char('6') => game.set_number(6),
                KeyCode::Char('7') => game.set_number(7),
                KeyCode::Char('8') => game.set_number(8),
                KeyCode::Char('9') => game.set_number(9),
                KeyCode::Delete | KeyCode::Backspace => game.delete_number(),
                KeyCode::Char('q') => {
                    break;
                }
                _ => {}
            }
        }
    }

    stop_sender.send(()).unwrap();
    draw_thread_handle.join().unwrap()?;
    Ok(())
}
