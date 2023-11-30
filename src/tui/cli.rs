use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::Paragraph, Terminal};
use std::{
    io, panic,
    time::{Duration, Instant},
};

use crate::actor::{Game, GameState};

use super::game;

pub fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        restore_terminal().unwrap();
        original_hook(panic);
    }));
}

pub async fn run(mut app: GameState) -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut last_tick = Instant::now();
    loop {
        let lock = app.lock().await;
        if render(&lock, &mut terminal).is_err() {
            break;
        }
        drop(lock);

        if tick(&mut app, &mut last_tick).await.is_err() {
            break;
        }
    }

    terminal.show_cursor()?;
    restore_terminal()?;
    Ok(())
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen)?;
    Ok(())
}

fn render(app: &Game, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    terminal.draw(|rect| {
        if let Some(game) = &app.game {
            let w: u16 = game.map_width.unsigned_abs();
            let h: u16 = game.map_height.unsigned_abs();
            rect.render_widget(game::render(app, w, h), Rect::new(0, 0, w, h));
            rect.render_widget(
                Paragraph::new(format!("{game:#?}")),
                Rect::new(0, h + 1, rect.size().width, rect.size().height - h - 1),
            );
        }
    })?;
    Ok(())
}

async fn tick(app: &mut GameState, last_tick: &mut Instant) -> anyhow::Result<()> {
    let tick_rate = Duration::from_millis(50);
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            app.lock().await.handle_input(key)?;
        }
    }

    if last_tick.elapsed() >= tick_rate {
        // app.lock().await.on_tick();
        *last_tick = Instant::now();
    }

    Ok(())
}
