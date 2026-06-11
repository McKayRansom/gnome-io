// use hecs::World;
use macroquad::{
    color::{Color, colors},
    math::{Rect, Vec2, vec2},
    shapes::draw_rectangle_lines,
};

use crate::{
    block::BlockInfoFlags,
    context::Context,
    entity::{self, BaseEntity, Entities, Entity, HIDDEN_FACTION, gnome::Gnome, goblin::Goblin},
    game::{Game, GameCtx},
    grid::{
        Grid, Pos,
        pos::{PIXEL_SIZE, dirs},
    },
    job::JobManager,
    text::{draw_text, draw_text_screen_centered},
    tile::{Content, ContentItem, TileBiome}, // tileset::{GRID_CELL_SIZE, PIXEL_SIZE, Sprite, pos_to_rect, sprites},
};

pub fn draw_game(game: &Game, ctx: &Context) {
    draw_tiles(&game.grid, &game.game_ctx, ctx, &game.entities);
    draw_managers(&game.job_manager, &game.game_ctx, ctx);
    draw_stocks(&game.grid, &game.game_ctx, ctx);
    draw_status(game, ctx);
}

fn draw_items(
    game_ctx: &GameCtx,
    ctx: &Context,
    items: &[ContentItem],
    start: &Rect,
    stack_spacing: f32,
) {
    let mut dest = start.clone();
    for item in items {
        // if let Content::Item(item) = item {
        ctx.tileset.draw_tile(
            if let Some(item) = game_ctx.items.get_info(&item.0) {
                &item.sprite
            } else {
                "unknown"
            },
            &dest,
            colors::WHITE,
        );
        dest.y -= stack_spacing * PIXEL_SIZE * ctx.camera.zoom;
    }
    // }
}

fn draw_tiles(grid: &Grid, game_ctx: &GameCtx, ctx: &Context, entities: &Entities) {
    let zoomed_out = ctx.camera.zoom < 0.25;
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let pos: Pos = (x, y).into();
            // skip invisible tiles
            let tile = grid.get_tile(pos).unwrap();
            // if tile.biome == TileBiome::Stone
            //     && !dirs::ALL.iter().any(|dir| {
            //         grid.get_tile(pos + *dir)
            //             .is_some_and(|tile| tile.is_passable())
            //     })
            // {
            //     continue;
            // }
            let dest: Rect = ctx.camera.to_screen_rect(pos.into());
            ctx.tileset.draw_tile(
                // "sky",
                match tile.biome {
                    TileBiome::Dirt => "grass",
                    TileBiome::Stone => "stone_floor",
                    TileBiome::Water => "water",
                    // _ => Sprite::new(0, 5),
                    TileBiome::Sky => "sky",
                },
                &dest,
                colors::WHITE,
            );

            // draw block first?
            if let Some(block) = tile.get_block() {
                let Some(block) = game_ctx.blocks.get_info(&block) else {
                    panic!("No block found fo id {}", block);
                };
                if block.solid() {
                    let non_solid_dirs: Vec<&Pos> = dirs::ALL
                        .iter()
                        .filter(|dir| {
                            grid.get_tile(pos + **dir).is_none_or(|tile| {
                                !tile.block_flags().contains(BlockInfoFlags::SOLID)
                            })
                        })
                        .collect();

                    // jank
                    ctx.tileset.draw_tile(
                        &if non_solid_dirs.len() == 0 && !game_ctx.debug.draw_hidden {
                            "stone_floor"
                        } else {
                            &block.sprite
                        },
                        &dest,
                        colors::WHITE,
                    );
                    if !zoomed_out {
                        for dir in non_solid_dirs {
                            ctx.tileset.draw_tile_rot(
                                "stone",
                                colors::WHITE,
                                &dest,
                                dirs::to_radians(*dir),
                            );
                        }
                    }
                } else {
                    ctx.tileset.draw_tile(&block.sprite, &dest, colors::WHITE);
                }
            }
            // then draw items
            let items: Vec<ContentItem> = tile
                .iter_content()
                .filter_map(|content| {
                    if let Content::Item(item) = content {
                        Some(*item)
                    } else {
                        None
                    }
                })
                .collect();
            draw_items(game_ctx, ctx, &items, &dest, 0.5);
        }
    }

    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let pos: Pos = (x, y).into();
            // skip invisible tiles
            let tile = grid.get_tile(pos).unwrap();

            // jobs
            for item in tile.iter_content() {
                if let Content::Job(job) = item {
                    ctx.tileset.draw_tile(
                        "mark",
                        &ctx.camera.to_screen_rect(pos.into()),
                        // &pos,
                        if let Some(job) = game_ctx.events.job_get(job) {
                            if job.in_progress {
                                Color::new(0., 0.6, 0.0, 1.0)
                                // Color::from_rgba(81, 255, 149, 255)
                            } else {
                                Color::new(0.6, 0.6, 0., 1.0)
                                // Color::from_rgba(250, 227, 51, 255)
                            }
                        } else {
                            Color::new(0.3, 0.0, 0., 1.0)
                            // Color::from_rgba(0.3, 0.0, 0., 1.0)
                        },
                    );
                }
            }

            // entities
            for item in tile.iter_content() {
                if let Content::Entity(gnome) = item {
                    // ctx.tileset.draw_tile(sprites, dest, color);
                    match entities.get(gnome.1).unwrap() {
                        Entity::Gnome(gnome) => draw_gnome(game_ctx, ctx, gnome),
                        Entity::Goblin(goblin) => draw_goblin(game_ctx, ctx, goblin),
                    }
                }
            }

            // draw debug info last last
            if let Some(faction) = game_ctx.debug.draw_pathable {
                if tile.is_passable(faction) {
                    draw_tile_outline(grid, &pos, colors::GREEN, ctx);
                }
            }
        }
    }
}

const ITEM_Y_DIFF: f32 = 15.0;

fn draw_goblin(game_ctx: &GameCtx, ctx: &Context, goblin: &Goblin) {
    // skip drawing hidden goblins
    if goblin.base.faction == HIDDEN_FACTION && !game_ctx.debug.draw_hidden {
        return;
    }
    let (flip, dest) = entity_draw_info(ctx, &goblin.base);

    let mut item_start = dest.clone();
    item_start.y -= PIXEL_SIZE * ITEM_Y_DIFF * ctx.camera.zoom;

    ctx.tileset
        .draw_tile_ex("goblin", colors::WHITE, &dest, flip);

    if goblin.fighting {
        ctx.tileset.draw_tile("fight", &item_start, colors::WHITE)
    }
}

fn draw_gnome(game_ctx: &GameCtx, ctx: &Context, gnome: &Gnome) {
    let (flip, dest) = entity_draw_info(ctx, &gnome.base);

    let mut item_start = dest.clone();
    item_start.y -= PIXEL_SIZE * ITEM_Y_DIFF * ctx.camera.zoom;

    let sprite = match gnome.status {
        entity::gnome::GnomeStatus::NONE => "gnome",
        entity::gnome::GnomeStatus::SLEEPING => "gnome_sleep",
        entity::gnome::GnomeStatus::EATING => "gnome_eat",
        entity::gnome::GnomeStatus::FIGHTING => "gnome",
    };
    ctx.tileset.draw_tile_ex(sprite, colors::WHITE, &dest, flip);

    match gnome.status {
        entity::gnome::GnomeStatus::NONE => {
            // I know I implemented all of this, but I actually don't like it, it's a bunch of extra noise
            // if gnome.base.is_tired() {
            //     ctx.tileset.draw_tile("think", &item_start, colors::WHITE);
            //     ctx.tileset.draw_tile("sleep", &item_start, colors::WHITE);
            // } else if gnome.base.is_hungry() {
            //     ctx.tileset.draw_tile("think", &item_start, colors::WHITE);
            //     ctx.tileset.draw_tile("bread", &item_start, colors::WHITE);
            // } else {
            draw_items(game_ctx, ctx, &gnome.base.items, &item_start, 0.5);
            // }
        }
        entity::gnome::GnomeStatus::SLEEPING => {
            ctx.tileset.draw_tile("sleep", &item_start, colors::WHITE)
        }
        entity::gnome::GnomeStatus::EATING => {}
        entity::gnome::GnomeStatus::FIGHTING => {
            ctx.tileset.draw_tile("fight", &item_start, colors::WHITE)
        }
    }
}

fn entity_draw_info(ctx: &Context, base: &BaseEntity) -> (bool, Rect) {
    let mut dest_rect: Rect = base.pos.into();
    let flip = base.dir == dirs::LEFT || base.dir == dirs::DOWN;

    if base.lag > 0 {
        let dir: Vec2 = base.dir.into();

        let offset = dir * (base.timer as f32 / base.lag as f32);
        dest_rect = dest_rect.offset(offset);
        // dbg!(gnome, offset);
    }
    let dest = ctx.camera.to_screen_rect(dest_rect);
    (flip, dest)
}

// use for debug info (it's ugly)
pub fn draw_rect_outline(rect: &Rect, color: Color, ctx: &Context) {
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        PIXEL_SIZE * ctx.camera.zoom * 2., /* WHY */
        color,
    );
}

pub fn draw_tile_outline(_grid: &Grid, pos: &Pos, color: Color, ctx: &Context) {
    let rect: Rect = (*pos).into();

    draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
    // }
}

fn draw_managers(manager: &JobManager, _game_ctx: &GameCtx, ctx: &Context) {
    for (pos, _) in manager.snow_manager.snow.iter() {
        // could also interpolate
        let rect: Rect = ctx.camera.to_screen_rect((*pos).into()).into();
        ctx.tileset.draw_tile("snow", &rect, colors::WHITE);
    }
    for (pos, _) in manager.farm_manager.farm_pos.iter() {
        let rect: Rect = ctx.camera.to_screen_rect((*pos).into()).into();
        ctx.tileset.draw_tile("farm_pos", &rect, colors::WHITE);
    }
    // NOTE: We could switch to drawing furnace_active here...
    for pos in manager.craft_manager.workshop_pos.iter() {
        let rect: Rect = ctx.camera.to_screen_rect((*pos).into()).into();
        ctx.tileset.draw_tile("farm_pos", &rect, colors::WHITE);
    }
}

fn draw_stocks(grid: &Grid, game_ctx: &GameCtx, ctx: &Context) {
    // this really shouldn't be random order but here we are
    let mut pos = vec2(10., 20.);
    draw_text(
        ctx,
        "Stocks:",
        pos.x,
        pos.y,
        crate::text::Size::Small,
        colors::WHITE,
    );
    pos.y += 30.;
    for (item, stock) in grid.stocks.iter() {
        draw_text(
            ctx,
            format!("{}: {}", game_ctx.items.get_info(item).unwrap().name, stock).as_str(),
            pos.x,
            pos.y,
            crate::text::Size::Small,
            colors::WHITE,
        );
        pos.y += 26.;
    }
}

fn draw_status(game: &Game, ctx: &Context) {
    let time = &game.game_ctx.time;
    draw_text_screen_centered(
        ctx,
        format!("Day {} of {:?} Year {}", time.day, time.season, time.year).as_str(),
        // ctx.screen_size.x - 200.,
        25.,
        crate::text::Size::Small,
        colors::WHITE,
    );
}
