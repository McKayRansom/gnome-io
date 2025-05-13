// use hecs::World;
use macroquad::color::{Color, colors};

use crate::{
    context::Context,
    game::{Game, GameCtx, Gnomes},
    grid::{Grid, Pos},
    job::{Job, JOB_QUEUE},
    tile::{Entity, TileBiome},
    tileset::{pos_to_rect, sprites, Sprite, GRID_CELL_SIZE, PIXEL_SIZE},
};

pub fn draw_game(game: &Game, ctx: &Context) {
    draw_tiles(&game.grid, &game.game_ctx, ctx);
    // draw_gnomes(&game.gnomes, ctx);
    draw_jobs(&game.game_ctx, &game.grid, &game.gnomes, ctx);
}

fn draw_tiles(grid: &Grid, game_ctx: &GameCtx, ctx: &Context) {
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let pos: Pos = (x, y).into();
            let tile = grid.get_tile(pos).unwrap();
            let dest = pos_to_rect(pos.into());
            ctx.tileset.draw_tile(
                match tile.biome {
                    TileBiome::Dirt => Sprite::new(1, 1),
                    TileBiome::Stone => Sprite::new(0, 2),
                    TileBiome::Water => Sprite::new(1, 2),
                    // _ => Sprite::new(0, 5),
                },
                &dest,
                colors::WHITE,
            );
            for item in tile.iter_entities() {
                ctx.tileset.draw_tile(
                    match item {
                        Entity::Item(item) => {
                            if let Some(item) = game_ctx.items.get_item(item) {
                                item.sprite
                            } else {
                                sprites::UNKOWN_ITEM
                            }
                        }
                        Entity::Gnome(_gnome) => sprites::GNOME,
                        Entity::Block(block) => {
                            let Some(block) = game_ctx.blocks.get_block(&block) else {
                                panic!("No block found fo id {}", block);
                            };
                            block.sprite
                        },
                        Entity::Job(_) => {
                            draw_tile_outline(grid, &pos, Color::new(0.3, 0.3, 0., 1.0), ctx);
                            continue;
                        }
                    },
                    &dest,
                    colors::WHITE,
                );
            }
        }
    }
}

pub const TILE_PERSPECTIVE_HEIGHT: f32 = PIXEL_SIZE * 6.;

pub fn draw_tile_outline(grid: &Grid, pos: &Pos, color: Color, ctx: &Context) {
    let mut rect = pos_to_rect(*pos);
    if grid.get_tile(*pos).is_some_and(|tile| !tile.is_passable()) {
        // draw "box" around block
        // "top side"
        rect.h = GRID_CELL_SIZE.0;
        rect.y -= TILE_PERSPECTIVE_HEIGHT;
        ctx.tileset.draw_rect_outline(&rect, color);
        // "front side" facing camera
        rect.h = TILE_PERSPECTIVE_HEIGHT;
        rect.y += GRID_CELL_SIZE.0;
        ctx.tileset.draw_rect_outline(&rect, color);
    } else {
        ctx.tileset.draw_rect_outline(&rect, color);
    }
}

fn draw_jobs(game_ctx: &GameCtx, grid: &Grid, gnomes: &Gnomes, ctx: &Context) {
    for event in game_ctx.events.get_queue(&JOB_QUEUE).unwrap().iter() {
        if let Some(job) = event.value.downcast_ref::<Job>() {
            draw_tile_outline(grid, &job.pos, Color::new(0.3, 0.3, 0., 1.0), ctx);
        }
    }
    // look for failed jobs
    for timer in game_ctx.events.timers.iter() {
        if let Some(job) = timer.event.as_ref().unwrap().value.downcast_ref::<Job>() {
            draw_tile_outline(grid, &job.pos, Color::new(0.3, 0.0, 0., 1.0), ctx);
        }
    }
    // draw in-progress jobs
    for gnome in gnomes.values() {
        if let Some(job) = &gnome.job {
            draw_tile_outline(grid, &job.pos, Color::new(0., 0.3, 0., 1.0), ctx);
        }
    }
}
