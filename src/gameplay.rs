use macroquad::{
    color::Color,
    input::{KeyCode, is_key_down, is_mouse_button_pressed, is_mouse_button_released, mouse_wheel},
    math::{Vec2, vec2},
    time::get_time,
    ui::{hash, root_ui, widgets::Window},
};

use crate::{
    block::{BlockId, blocks},
    context::Context,
    draw::{draw_game, draw_tile_outline},
    game::{Game, GameSpeed, time::GameTimeEvent},
    grid::{Pos, pos::GRID_CELL_SIZE},
    tile::Content,
    ui::{
        menu::{Menu, MenuItem},
        popup::{Popup, PopupResult},
        toolbar::{TOOLBAR_SPACE, Toolbar, ToolbarItem},
    },
};

pub enum GameAction {
    Mine,
    Build,
    Farm,
    Cancel,
}

#[derive(PartialEq, Eq)]
pub enum TimeSelect {
    Pause,
    FastForward,
    Menu,
}

enum PauseMenuSelect {
    Continue,
    Save,
    Load,
    Quit,
    // Restart,
}

pub struct Gameplay {
    game: Game,
    mouse_down_pos: Option<Pos>,
    draw_details_pos: Option<Pos>,
    action_toolbar: Toolbar<GameAction>,
    build_toolbar: Toolbar<BlockId>,
    time_select: Toolbar<TimeSelect>,
    popup: Option<Popup>,
    pause_menu: Menu<PauseMenuSelect>,
}

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.05;
// const PLUS_MINUS_SENSITVITY: f32 = 0.8; // 20% zoom seems pretty standard (I.E. that is also what VSCode does)

impl Gameplay {
    pub fn new(ctx: &mut Context) -> Self {
        ctx.camera.change_zoom(0.2);
        ctx.camera.camera = vec2(-100., 300.);
        Self {
            game: Game::generate(get_time()),
            mouse_down_pos: None,
            draw_details_pos: None,
            action_toolbar: Toolbar::new(
                crate::ui::toolbar::ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(GameAction::Mine, "Mine stuff", '1', "mine".into()),
                    ToolbarItem::new(GameAction::Build, "Build stuff", '2', "build".into()),
                    ToolbarItem::new(GameAction::Farm, "Farm stuff", '3', "farm".into()),
                    ToolbarItem::new(GameAction::Cancel, "Cancel stuff", '4', "cancel".into()),
                ],
            ),
            build_toolbar: Toolbar::new(
                crate::ui::toolbar::ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(blocks::STONE_BLOCK_ID, "Stone wall", '1', "stone".into()),
                    ToolbarItem::new(
                        blocks::CRAFT_TABLE_ID,
                        "Crafting table",
                        '2',
                        "craft_table".into(),
                    ),
                    ToolbarItem::new(blocks::FURNACE_ID, "Furnace", '3', "furnace".into()),
                    ToolbarItem::new(blocks::BED_ID, "Bed", '4', "bed".into()),
                ],
            ),
            time_select: Toolbar::new(
                crate::ui::toolbar::ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(TimeSelect::Pause, "Pause game", ' ', "pause".into()),
                    // ToolbarItem::new(TimeSelect::Play, "Play game", ' ', Sprite::new(10, 1)),
                    ToolbarItem::new(
                        TimeSelect::FastForward,
                        "Fast Forward game",
                        ' ',
                        "fast_forward".into(),
                    ),
                    ToolbarItem::new(TimeSelect::Menu, "Menu", '\u{1b}', "menu".into()),
                ],
            ),
            popup: None,
            pause_menu: Menu::new(vec![
                MenuItem::new(PauseMenuSelect::Continue, "Close".to_string()),
                MenuItem::new(PauseMenuSelect::Save, "Save".to_string()),
                MenuItem::new(PauseMenuSelect::Load, "Load".to_string()),
                MenuItem::new(PauseMenuSelect::Quit, "Menu".to_string()),
                // MenuItem::new(PauseMenuSelect::Restart, "Restart".to_string()),
            ]),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        while self.game.should_update(get_time()) {
            match self.game.update() {
                GameTimeEvent::None => {}
                GameTimeEvent::YearEnd => {
                    self.popup = Some(Popup::new(format!(
                        "You survived Year {}!",
                        self.game.game_ctx.time.year - 1
                    )));
                }
            }
        }
        if self.game.entities.len() == 0 {
            self.popup = Some(Popup::new(format!(
                "Game over, you survived until {:?} Year {}",
                self.game.game_ctx.time.season, self.game.game_ctx.time.year
            )));
        }

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

        self.time_select
            .draw(ctx, ctx.screen_size.x - TOOLBAR_SPACE * 1.5, 0.);
        match self.time_select.get_selected() {
            Some(TimeSelect::Pause) => {
                self.game.speed = GameSpeed::Paused;
                self.time_select.items[0].sprite = "play".into();
            }
            Some(TimeSelect::FastForward) => {
                self.time_select.items[0].sprite = "play".into();
                self.game.speed = GameSpeed::FastForward;
            }
            Some(TimeSelect::Menu) => {
                self.time_select.items[0].sprite = "play".into();
                self.game.speed = GameSpeed::Paused;
                if let Some(selected) = self.pause_menu.draw(hash!()) {
                    match selected {
                        PauseMenuSelect::Continue => {
                            self.time_select.clear_selected();
                        }
                        PauseMenuSelect::Save => {
                            self.game.save().expect("Failed to save!");
                            // println!("SAVED GAME");
                        }
                        PauseMenuSelect::Load => {
                            self.game = Game::load().expect("Failed to load");
                            // self.game.save().expect("Failed to save!");
                            // println!("SAVED GAME");
                        }
                        PauseMenuSelect::Quit => {
                            // ctx.switch_scene_to = Some(crate::scene::EScene::MainMenu)
                        } // PauseMenuSelect::Restart => {
                          // ctx.switch_scene_to = Some(crate::scene::EScene::Gameplay(Box::new(
                          //     new_level(map.metadata.level_number),
                          // )))
                          // }
                    }
                }
            }
            None => {
                self.game.speed = GameSpeed::Normal;
                self.time_select.items[0].sprite = "pause".into();
            }
        }

        // if let Some(chr) = ctx.key_pressed {

        // }

        if let Some(draw_details_pos) = self.draw_details_pos {
            // special behaviour for workshops
            let size = Vec2::new(200., 200.);
            if Window::new(
                hash!(),
                ctx.camera.to_screen(
                    Into::<Vec2>::into(draw_details_pos) + vec2(GRID_CELL_SIZE.0 / 2., 0.),
                ) - vec2(size.x / 2., size.y),
                size,
            )
            .titlebar(false)
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                let tile = self.game.grid.get_tile(draw_details_pos).unwrap();

                // remove workshop menu for now...
                // let workshops: Vec<BlockId> = vec![CRAFT_TABLE_ID, FURNACE_ID];
                // if tile
                //     .get_block()
                //     .is_some_and(|block| workshops.contains(&block))
                // {
                //     let workshop_block = tile.get_block().unwrap();
                //     // show recipes instead
                //     for (item_id, item) in self.game.game_ctx.items.iter_items() {
                //         if item
                //             .recipe
                //             .as_ref()
                //             .is_some_and(|recipe| recipe.0 == workshop_block)
                //         {
                //             if ui.button(None, format!("{:?}", item.name).as_str()) {
                //                 // make this recipe!
                //                 JobManager::create_job(
                //                     &mut self.game.grid,
                //                     &mut self.game.game_ctx.events,
                //                     Job::new(
                //                         draw_details_pos,
                //                         CRAFTING_TIME,
                //                         Some(Content::Item(*item_id)),
                //                         item.recipe.as_ref().unwrap().1.clone(),
                //                     ),
                //                 );
                //             }
                //         }
                //     }
                // } else {
                for item in tile.iter_entities() {
                    ui.label(
                        None,
                        format!(
                            "{:?}",
                            if let Content::Item(item) = item {
                                self.game.game_ctx.items.get_item(item).unwrap().name
                            } else {
                                ""
                            }
                        )
                        .as_str(),
                    );
                }
                // }
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
                    dbg!(self.game.grid.get_tile(mouse_pos).unwrap());
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

        if let Some(popup) = &self.popup {
            self.game.speed = GameSpeed::Paused;
            match popup.draw() {
                Some(PopupResult::Ok) => {
                    self.popup = None;
                    // let level_number = self.map.metadata.level_number + 1;
                    // ctx.switch_scene_to = if level_number < LEVEL_COUNT {
                    //     Some(EScene::Gameplay(Box::new(new_level(level_number))))
                    // } else {
                    //     Some(EScene::MainMenu)
                    // }
                }
                Some(PopupResult::Cancel) => {
                    self.popup = None;
                    // self.map.metadata.level_complete = true;
                }
                None => {}
            }
        }
    }
}
