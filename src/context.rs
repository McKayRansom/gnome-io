use macroquad::{input::{get_char_pressed, mouse_position}, math::Vec2};

use crate::tileset::Tileset;

pub struct Context {
    pub tileset: Tileset,
    pub key_pressed: Option<char>,
    pub mouse_pos: Option<Vec2>,
}

impl Context {
    pub async fn new() -> Self {
        Self {
            tileset: Tileset::new().await,
            key_pressed: None,
            mouse_pos: None,
        }
    }

    pub fn update(&mut self) {
        self.mouse_pos = Some(mouse_position().into());
        self.key_pressed = get_char_pressed();
    }
}
