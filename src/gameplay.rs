use macroquad::{
    color::Color,
    input::{KeyCode, is_key_down, is_mouse_button_pressed, is_mouse_button_released, mouse_wheel},
    math::vec2,
    ui::{hash, root_ui, widgets::Window},
};
use quad_lib::tileset::Sprite;

use crate::{
    block::BlockId,
    context::Context,
    draw::{draw_game, draw_tile_outline, sprites},
    game::{Game, CRAFT_TABLE_ID, FURNACE_ID, STONE_BLOCK_ID},
    grid::Pos,
    job::{Job, JobManager},
    tile::Entity,
    toolbar::{Toolbar, ToolbarItem, TOOLBAR_SPACE},
};

pub enum GameAction {
    Mine,
    Build,
    Farm,
    Cancel,
}

pub struct Gameplay {
    game: Game,
    mouse_down_pos: Option<Pos>,
    draw_details_pos: Option<Pos>,
    action_toolbar: Toolbar<GameAction>,
    build_toolbar: Toolbar<BlockId>,
}

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.05;
// const PLUS_MINUS_SENSITVITY: f32 = 0.8; // 20% zoom seems pretty standard (I.E. that is also what VSCode does)

impl Gameplay {
    pub fn new(ctx: &mut Context) -> Self {
        ctx.camera.change_zoom(0.9);
        ctx.camera.camera = vec2(500., 500.);
        Self {
            game: Game::generate(),
            mouse_down_pos: None,
            draw_details_pos: None,
            action_toolbar: Toolbar::new(crate::toolbar::ToolbarType::Horizontal, vec![
                ToolbarItem::new(GameAction::Mine, "Mine stuff", '1', Sprite::new(3, 0)),
                ToolbarItem::new(GameAction::Build, "Build stuff", '2', Sprite::new(3, 1)),
                ToolbarItem::new(GameAction::Farm, "Farm stuff", '3', Sprite::new(3, 2)),
                ToolbarItem::new(GameAction::Cancel, "Cancel stuff", '4', Sprite::new(3, 3)),
            ]),
            build_toolbar: Toolbar::new(crate::toolbar::ToolbarType::Horizontal, vec![
                ToolbarItem::new(STONE_BLOCK_ID, "Stone wall", '1', sprites::STONE),
                ToolbarItem::new(CRAFT_TABLE_ID, "Crafting table", '2', sprites::CRAFT_TABLE),
                ToolbarItem::new(FURNACE_ID, "Furnace", '3', sprites::FURNACE),
            ]),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.game.update();

        // check WASD
        // TODO: Right click PAN
        if is_key_down(KeyCode::W) {
            ctx.camera.camera.y -= WASD_MOVE_SENSITIVITY / ctx.camera.zoom;
        }
        if is_key_down(KeyCode::A) {
            ctx.camera.camera.x -= WASD_MOVE_SENSITIVITY / ctx.camera.zoom;
        }
        if is_key_down(KeyCode::S) {
            ctx.camera.camera.y += WASD_MOVE_SENSITIVITY / ctx.camera.zoom;
        }
        if is_key_down(KeyCode::D) {
            ctx.camera.camera.x += WASD_MOVE_SENSITIVITY / ctx.camera.zoom;
        }

        if ctx.mouse_pos.is_none() {
            return;
        }

        let new_mouse_wheel = mouse_wheel();
        if new_mouse_wheel.1 != 0. {
            ctx.camera
                .change_zoom(SCROLL_SENSITIVITY * new_mouse_wheel.1);
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        draw_game(&self.game, ctx);

        self.action_toolbar.draw(
            ctx,
            ctx.screen_size.x / 2.0,
            ctx.screen_size.y - TOOLBAR_SPACE,
        );

        if matches!(self.action_toolbar.get_selected(), Some(GameAction::Build)) {
            self.build_toolbar.draw(
                ctx,
                ctx.screen_size.x / 2.0,
                ctx.screen_size.y - TOOLBAR_SPACE * 2.,
            );
        }

        if let Some(draw_details_pos) = self.draw_details_pos {
            // special behaviour for workshops
            if Window::new(
                hash!(),
                ctx.camera.to_screen(draw_details_pos.into()) - vec2(0., 100.),
                vec2(100., 100.),
            )
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                let tile = self.game.grid.get_tile(draw_details_pos).unwrap();
                let workshops: Vec<BlockId> = vec![CRAFT_TABLE_ID, FURNACE_ID];
                if tile.get_block().is_some_and(|block| workshops.contains(&block)) {
                    let workshop_block = tile.get_block().unwrap();
                    // show recipes instead
                    for (item_id, item) in self.game.game_ctx.items.iter_items() {
                        if item
                            .recipe
                            .as_ref()
                            .is_some_and(|recipe| recipe.0 == workshop_block)
                        {
                            if ui.button(None, format!("{:?}", item_id).as_str()) {
                                // make this recipe!
                                JobManager::create_job(
                                    &mut self.game.grid,
                                    &mut self.game.game_ctx.events,
                                    Job::new(
                                        draw_details_pos,
                                        30,
                                        Some(Entity::Item(*item_id)),
                                        item.recipe.as_ref().unwrap().1.clone(),
                                    ),
                                );
                            }
                        }
                    }
                } else {
                    for item in tile.iter_entities() {
                        ui.label(None, format!("{:?}", item).as_str());
                    }
                }
            }) == false
            {
                log::info!("UI RETURNED FALSE?");
            }
        }

        if let Some(mouse_pos) = ctx.mouse_pos {
            // if let Some(pos) =
            let mouse_pos = ctx.camera.to_world(mouse_pos.into()).into();
            if self.game.grid.is_valid_pos(mouse_pos) {
                if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    self.mouse_down_pos = Some(mouse_pos);
                }
                if is_mouse_button_released(macroquad::input::MouseButton::Left) {
                    for pos in self.mouse_down_pos.unwrap_or(mouse_pos).iter(mouse_pos) {
                        if let Some(action) = self.action_toolbar.get_selected() {
                            match action {
                                GameAction::Mine => self.game.mine(pos),
                                GameAction::Build => {
                                    if let Some(block_id) = self.build_toolbar.get_selected() {
                                        self.game.build(pos, *block_id)
                                    }
                                }
                                GameAction::Farm => self.game.farm(pos),
                                GameAction::Cancel => self.game.cancel(pos),
                            }
                        } else {
                            self.draw_details_pos = if self.draw_details_pos == Some(pos) {
                                None
                            } else {
                                Some(pos)
                            };
                        }
                    }
                    self.mouse_down_pos = None;
                }
            } else {
                self.mouse_down_pos = None;
            }

            // draw_selected
            for pos in self.mouse_down_pos.unwrap_or(mouse_pos).iter(mouse_pos) {
                draw_tile_outline(&self.game.grid, &pos, Color::new(1., 1., 1., 0.3), ctx);
            }
        }
    }
}
