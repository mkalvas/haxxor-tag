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
        .x_bounds(bounds(width))
        .y_bounds(bounds(height))
        .marker(Marker::Bar)
        .paint(|ctx| {
            ctx.draw(state);
        })
        .block(game_block())
}

fn bounds(span: u16) -> [f64; 2] {
    let half = (span / 2) as f64;
    [-half, if span % 2 == 0 { half } else { half + 1_f64 }]
}

pub fn game_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(" XOR Tag ")
        .style(Style::default().fg(Color::White))
}
