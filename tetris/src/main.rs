mod game;
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

use game::{Game, GameState};

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
                KeyCode::Up => game.rotate(),
                KeyCode::Char(' ') => game.hard_drop(),
                KeyCode::Esc => match game.game_state {
                    GameState::Paused => game.toggle_paused(),
                    GameState::Playing => game.toggle_paused(),
                    GameState::GameOver => game.reset(),
                },
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
