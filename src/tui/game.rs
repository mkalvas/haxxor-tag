use ratatui::{
    style::{Color, Style},
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context},
        Block, Borders,
    },
};

use crate::actor_v2::Game;

pub fn render(state: &Game, width: u16, height: u16) -> Canvas<impl Fn(&mut Context<'_>) + '_> {
    Canvas::default()
        .x_bounds([0f64, width.into()])
        .y_bounds([0f64, height.into()])
        .marker(Marker::HalfBlock)
        .paint(|ctx| {
            ctx.draw(state);
        })
        .block(game_block())
}

pub fn game_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(" XOR Tag ")
        .style(Style::default().fg(Color::White))
}
