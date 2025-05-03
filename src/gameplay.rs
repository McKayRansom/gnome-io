use macroquad::{
    color::Color,
    input::{is_mouse_button_pressed, mouse_position},
};

use crate::{
    context::Context,
    draw::{draw_game, pos_to_rect},
    game::Game,
    grid::Pos,
    job::Job,
    tile::Tile,
};

pub struct Gameplay {
    game: Game,
    mouse_pos: Option<Pos>,
}

impl Gameplay {
    pub fn new() -> Self {
        Self {
            game: Game::generate(),
            mouse_pos: None,
        }
    }

    pub fn update(&mut self, context: &Context) {
        self.game.update();
        // if let Some(pos) =
        let mouse_pos = context.tileset.from_screen(mouse_position());
        if self.game.grid.is_valid_pos(mouse_pos) {
            self.mouse_pos = Some(mouse_pos);
            if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                self.game.mine(mouse_pos);
            }
        } else {
            self.mouse_pos = None;
        }
    }

    pub fn draw(&mut self, context: &Context) {
        draw_game(&self.game, context);
        if let Some(mouse_pos) = self.mouse_pos {
            context
                .tileset
                .draw_rect(&pos_to_rect(mouse_pos), Color::new(1., 1., 1., 0.3));
        }
    }
}
