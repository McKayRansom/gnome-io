use macroquad::{
    input::{get_char_pressed, is_key_pressed, mouse_position},
    math::Vec2,
    text::Font,
    ui::root_ui,
    window::{screen_height, screen_width},
};
use nanoserde::{DeRon, DeRonState};
use quad_lib::{camera::Camera, tileset::Tileset};

pub struct Context {
    pub font: Font,
    pub camera: Camera,
    pub tileset: Tileset,
    pub key_pressed: Option<char>,
    pub mouse_pos: Option<Vec2>,
    pub screen_size: Vec2,
}

impl Context {
    pub async fn new() -> Self {
        Self {
            font: crate::ui::skin::init().await,
            camera: Camera::new(),
            tileset: Self::load_tieset().await,
            key_pressed: None,
            mouse_pos: None,
            screen_size: Vec2::new(0., 0.),
        }
    }

    async fn load_tieset() -> Tileset {
        
        Tileset::new("assets/data/tileset.ron").await
    }

    pub async fn update(&mut self) {
        if is_key_pressed(macroquad::input::KeyCode::F5) {
            self.tileset = Self::load_tieset().await;
        }
        let mouse_pos = mouse_position();
        if !root_ui().is_mouse_over(mouse_pos.into()) {
            self.mouse_pos = Some(mouse_position().into());
        } else {
            self.mouse_pos = None;
        }
        self.key_pressed = get_char_pressed();
        self.screen_size = Vec2::new(screen_width(), screen_height());
    }
}
