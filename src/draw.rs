// use hecs::World;
use macroquad::{
    color::{Color, colors},
    math::Rect,
};

use crate::{
    context::Context,
    game::{Game, GameCtx},
    grid::{Grid, Pos},
    job::{Job, JOB_QUEUE},
    tile::TileBiome,
    tileset::Sprite,
};

pub fn draw_game(game: &Game, ctx: &Context) {
    draw_tiles(&game.grid, &game.game_ctx, ctx);
    // draw_gnomes(&game.gnomes, ctx);
    draw_jobs(&game.game_ctx, &game.grid, ctx);
}

fn draw_tiles(grid: &Grid, game_ctx: &GameCtx, ctx: &Context) {
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let pos: Pos = (x, y).into();
            let tile = grid.get_tile(pos).unwrap();
            ctx.tileset.draw_tile(
                if let Some(block) = tile.block {
                    let Some(block) = game_ctx.blocks.get_block(&block) else {
                        panic!("No block found fo id {}", block);
                    };
                    block.sprite
                } else {
                    match tile.biome {
                        TileBiome::Dirt => Sprite::new(1, 1),
                        TileBiome::Stone => Sprite::new(0, 2),
                        TileBiome::Water => Sprite::new(1, 2),
                        // _ => Sprite::new(0, 5),
                    }
                },
                &pos_to_rect((x, y).into()),
                colors::WHITE,
                0.,
            );
            if let Some(_gnome) = tile.gnome {
                ctx.tileset
                    .draw_tile(Sprite::new(0, 0), &pos_to_rect(pos), colors::WHITE, 0.);
            }
        }
    }
}

pub fn draw_tile_outline(grid: &Grid, pos: &Pos, color: Color, ctx: &Context) {
    let mut rect = pos_to_rect(*pos);
    if grid.get_tile(*pos).is_some_and(|tile| !tile.is_passable()) {
        // draw "box" around block
        // "top side"
        rect.h = GRID_CELL_SIZE.0;
        rect.y -= GRID_CELL_SIZE.0 / 2.;
        ctx.tileset.draw_rect_outline(&rect, color);
        // "front side" facing camera
        rect.h = GRID_CELL_SIZE.0 / 2.;
        rect.y += GRID_CELL_SIZE.0;
        ctx.tileset.draw_rect_outline(&rect, color);
    } else {
        ctx.tileset.draw_rect_outline(&rect, color);
    }
}

fn draw_jobs(game_ctx: &GameCtx, grid: &Grid, ctx: &Context) {
    for event in game_ctx.events.get_queue(&JOB_QUEUE).unwrap().iter() {
        if let Some(job) = event.value.downcast_ref::<Job>() {
        // for pos in &jobs.mine_manager.tiles_queued {
            draw_tile_outline(grid, &job.pos, Color::new(1., 1., 0., 0.5), ctx);
        // }


        }
        // for pos in &jobs.mine_manager.tiles_in_progress {
        //     draw_tile_outline(grid, pos, Color::new(0., 1., 0., 0.5), ctx);
        // }
    }
}

// Default zoom pixel size of Position
pub const GRID_CELL_SIZE: (f32, f32) = (64., 64.);
pub const PIXEL_SIZE: f32 = 64. / 8.;

pub fn pos_to_rect(pos: Pos) -> Rect {
    Rect::new(
        pos.x as f32 * GRID_CELL_SIZE.0,
        pos.y as f32 * GRID_CELL_SIZE.1, /* - (pos.z as f32 * GRID_Z_OFFSET) */
        GRID_CELL_SIZE.0,
        GRID_CELL_SIZE.1,
    )
}
