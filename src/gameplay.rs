use macroquad::{
    color::Color,
    input::{
        KeyCode, is_key_down, is_mouse_button_pressed, mouse_wheel,
    },
    window::{screen_height, screen_width},
};

use crate::{
    context::Context,
    draw::{draw_game, pos_to_rect},
    game::Game,
    grid::Pos,
    tileset::Sprite,
    toolbar::{TOOLBAR_SPACE, Toolbar, ToolbarItem},
};

pub enum GameAction {
    Mine,
    Build,
    Farm,
    Cancel,
}

pub struct Gameplay {
    game: Game,
    mouse_pos: Option<Pos>,
    action_toolbar: Toolbar<GameAction>,
}

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.05;
// const PLUS_MINUS_SENSITVITY: f32 = 0.8; // 20% zoom seems pretty standard (I.E. that is also what VSCode does)

impl Gameplay {
    pub fn new(ctx: &mut Context) -> Self {
        ctx.tileset.change_zoom(0.9);
        ctx.tileset.camera = (500., 500.);
        Self {
            game: Game::generate(),
            mouse_pos: None,
            action_toolbar: Toolbar::new(crate::toolbar::ToolbarType::Horizontal, vec![
                ToolbarItem::new(GameAction::Mine, "Mine stuff", '1', Sprite::new(3, 0)),
                ToolbarItem::new(GameAction::Build, "Build stuff", '2', Sprite::new(3, 1)),
                ToolbarItem::new(GameAction::Farm, "Farm stuff", '3', Sprite::new(3, 2)),
                ToolbarItem::new(GameAction::Cancel, "Cancel stuff", '4', Sprite::new(3, 3)),
            ]),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.game.update();

        // check WASD
        // TODO: Right click PAN
        if is_key_down(KeyCode::W) {
            ctx.tileset.camera.1 -= WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }
        if is_key_down(KeyCode::A) {
            ctx.tileset.camera.0 -= WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }
        if is_key_down(KeyCode::S) {
            ctx.tileset.camera.1 += WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }
        if is_key_down(KeyCode::D) {
            ctx.tileset.camera.0 += WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }

        let new_mouse_wheel = mouse_wheel();
        if new_mouse_wheel.1 != 0. {
            ctx.tileset
                .change_zoom(SCROLL_SENSITIVITY * new_mouse_wheel.1);
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        draw_game(&self.game, ctx);

        self.action_toolbar
            .draw(ctx, screen_width() / 2.0, screen_height() - TOOLBAR_SPACE);

        if let Some(mouse_pos) = ctx.mouse_pos {
            // if let Some(pos) =
            let mouse_pos = ctx.tileset.from_screen(mouse_pos.into());
            if self.game.grid.is_valid_pos(mouse_pos) {
                self.mouse_pos = Some(mouse_pos);
                if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    self.game.mine(mouse_pos);
                }
            } else {
                self.mouse_pos = None;
            }

            // draw_selected
            ctx.tileset
                .draw_rect(&pos_to_rect(mouse_pos), Color::new(1., 1., 1., 0.3));
        }
    }
}
