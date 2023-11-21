use ratatui::{
    style::{Color, Style},
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context},
        Block, Borders,
    },
};

use crate::actor::ActorState;

pub fn render(
    state: &ActorState,
    width: u16,
    height: u16,
) -> Canvas<impl Fn(&mut Context<'_>) + '_> {
    Canvas::default()
        .x_bounds([0f64, width.try_into().expect("negative width")])
        .y_bounds([0f64, height.try_into().expect("negative height")])
        .marker(Marker::Bar)
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
