use macroquad::{
    color::{Color, colors},
    input::is_mouse_button_pressed,
    math::{Rect, Vec2},
    shapes::draw_rectangle,
    window::screen_height,
};
use rustc_hash::FxHashMap;

use crate::{
    context::Context,
    entity::{Entity, EntityId, gnome::GnomeProfession},
    game::Game,
    text::{draw_text, measure_text},
};

#[derive(Default)]
pub struct Labor {
    _selected: usize,
}

const COLUMN_SPACING: Vec2 = Vec2::new(70.0, 15.0);

const PROFFESIONS: [GnomeProfession; 7] = [
    GnomeProfession::NONE,
    GnomeProfession::CRAFTING,
    GnomeProfession::BUILDING,
    GnomeProfession::MINING,
    GnomeProfession::FARMING,
    GnomeProfession::FIGHTING,
    GnomeProfession::CHILDING,
];

const PROFFESIONS_NAME: [&str; 7] = ["ANY", "CRAFT", "BUILD", "MINE", "FARM", "FIGHT", "CHILD"];

const SELECTED_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.2);

struct LaborAction {
    proffesion: GnomeProfession,
    add: bool,
}

impl Labor {
    fn make_labor_map(&self, game: &Game) -> FxHashMap<GnomeProfession, Vec<EntityId>> {
        let mut map = FxHashMap::default();

        for (id, entity) in game.entities.iter() {
            if let Entity::Gnome(gnome) = entity {
                map.entry(gnome.get_profession())
                    .or_insert(Vec::new())
                    .push(*id);
            }
        }

        map
    }

    pub fn draw_labor(&mut self, game: &mut Game, ctx: &mut Context) {
        let mut rect = Rect::new(0.0, screen_height() - 5.0, 30.0, 30.0);

        let labor_map = self.make_labor_map(game);

        let mut action: Option<LaborAction> = None;

        for i in 0..PROFFESIONS.len() {
            let proffesion = PROFFESIONS[i];
            let mut col_rect = rect.clone();
            draw_text(
                ctx,
                PROFFESIONS_NAME[i],
                col_rect.x,
                col_rect.y,
                crate::text::Size::Small,
                colors::WHITE,
            );
            let measure = measure_text(ctx, PROFFESIONS_NAME[i], crate::text::Size::Small);
            let text_rect = Rect::new(
                col_rect.x - 10.0,
                col_rect.y - measure.height - 10.0,
                measure.width + 20.0,
                measure.height + 20.0,
            );
            // selected text
            if ctx
                .mouse_pos
                .is_some_and(|mouse_pos| text_rect.contains(mouse_pos))
            {
                // draw selected
                draw_rectangle(
                    text_rect.x,
                    text_rect.y,
                    text_rect.w,
                    text_rect.h,
                    SELECTED_COLOR,
                );
                ctx.mouse_pos.take();
                if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    action = Some(LaborAction {
                        proffesion,
                        add: true,
                    })
                }
                if is_mouse_button_pressed(macroquad::input::MouseButton::Right) {
                    action = Some(LaborAction {
                        proffesion,
                        add: false,
                    })
                }
            }
            col_rect.y -= 60.0;
            for enttiy in labor_map.get(&proffesion).unwrap_or(&Vec::new()) {
                if let Entity::Gnome(gnome) = game.entities.get(*enttiy).unwrap() {
                    if gnome.get_profession() == proffesion {
                        ctx.tileset
                            .draw_tile(if gnome.has_job() {"labor_full"} else {"labor_empty"}, &col_rect, colors::WHITE);
                        if ctx
                            .mouse_pos
                            .is_some_and(|mouse_pos| col_rect.contains(mouse_pos))
                        {
                            // draw selected
                            draw_rectangle(
                                col_rect.x,
                                col_rect.y,
                                col_rect.w,
                                col_rect.h,
                                SELECTED_COLOR,
                            );
                            ctx.mouse_pos.take();
                        }
                        col_rect.y -= COLUMN_SPACING.y;
                    }
                }
            }

            rect.x += COLUMN_SPACING.x;
        }

        if let Some(action) = action {
            let proffesion = if action.add {
                GnomeProfession::NONE
            } else {
                action.proffesion
            };
            if let Some(vec) = labor_map.get(&proffesion) {
                if let Some(id) = vec.first() {
                    let Some(Entity::Gnome(gnome)) = game.entities.get_mut(*id) else {
                        panic!("Fault in labor logic!")
                    };
                    gnome.set_profession(
                        if action.add {
                            action.proffesion
                        } else {
                            GnomeProfession::NONE
                        },
                        &mut game.game_ctx.events,
                    );
                }
            }
        }
    }
}
