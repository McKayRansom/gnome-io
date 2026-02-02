// use hecs::World;
use macroquad::{
    color::{Color, colors},
    math::{Rect, Vec2, vec2},
    shapes::draw_rectangle_lines,
};

use crate::{
    context::Context,
    game::{Game, GameCtx, Gnomes},
    grid::{
        Grid, Pos,
        pos::{PIXEL_SIZE, dirs},
    },
    text::{draw_text, draw_text_screen_centered},
    tile::{Content, TileBiome}, // tileset::{GRID_CELL_SIZE, PIXEL_SIZE, Sprite, pos_to_rect, sprites},
};

pub fn draw_game(game: &Game, ctx: &Context) {
    draw_tiles(&game.grid, &game.game_ctx, ctx, &game.gnomes);
    draw_stocks(&game.grid, &game.game_ctx, ctx);
    draw_status(game, ctx);
}

fn draw_tiles(grid: &Grid, game_ctx: &GameCtx, ctx: &Context, gnomes: &Gnomes) {
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
                match tile.biome {
                    TileBiome::Dirt => "grass",
                    TileBiome::Stone => "stone_floor",
                    TileBiome::Water => "water",
                    // _ => Sprite::new(0, 5),
                },
                &dest,
                colors::WHITE,
            );

            // draw block first?
            if let Some(block) = tile.get_block() {
                let Some(block) = game_ctx.blocks.get_block(&block) else {
                    panic!("No block found fo id {}", block);
                };
                ctx.tileset.draw_tile(&block.sprite, &dest, colors::WHITE);
            }
            // then draw items
            for item in tile.iter_entities() {
                if let Content::Item(item) = item {
                    ctx.tileset.draw_tile(
                        if let Some(item) = game_ctx.items.get_item(item) {
                            &item.sprite
                        } else {
                            "unknown"
                        },
                        &dest,
                        colors::WHITE,
                    );
                }
            }
        }
    }
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let pos: Pos = (x, y).into();
            // skip invisible tiles
            let tile = grid.get_tile(pos).unwrap();
            // then gnomes
            for item in tile.iter_entities() {
                if let Content::Gnome(gnome) = item {
                    // ctx.tileset.draw_tile(sprites, dest, color);
                    let gnome = gnomes.get(gnome).unwrap();
                    let _think_box: Rect = ctx.camera.to_screen_rect((pos + dirs::UP).into());

                    // oh g oh f
                    let mut dest_rect: Rect = gnome.pos.into();
                    let flip = gnome.dir == dirs::LEFT || gnome.dir == dirs::DOWN;

                    if gnome.lag > 0 {
                        let dir: Vec2 = gnome.dir.into();

                        let offset = dir * (gnome.timer as f32 / gnome.lag as f32);
                        dest_rect = dest_rect.offset(offset);
                        // dbg!(gnome, offset);
                    }
                    let dest = ctx.camera.to_screen_rect(dest_rect);
                    // let diff =

                    // if gnome.tired < SLEEP_TIRED {
                    //     ctx.tileset
                    //         .draw_tile(sprites::THINK, &think_box, colors::WHITE);
                    //     ctx.tileset
                    //         .draw_tile(sprites::BED, &think_box, colors::WHITE);
                    // } else if gnome.sleeping {
                    //     ctx.tileset
                    //         .draw_tile(sprites::THINK, &think_box, colors::WHITE);
                    //     ctx.tileset
                    //         .draw_tile(sprites::SLEEP, &think_box, colors::WHITE);
                    // } else if gnome.food < gnome::FOOD_EAT {
                    //     ctx.tileset
                    //         .draw_tile(sprites::THINK, &think_box, colors::WHITE);
                    //     ctx.tileset
                    //         .draw_tile(sprites::BREAD, &think_box, colors::WHITE);
                    // }
                    ctx.tileset
                        .draw_tile_ex("gnome", colors::WHITE, &dest, flip);
                }
            }

            // draw jobs last (on top)
            for item in tile.iter_entities() {
                if let Content::Job(job) = item {
                    draw_tile_outline(
                        grid,
                        &pos,
                        if let Some(job) = game_ctx.events.jobs.get(job) {
                            if job.in_progress {
                                Color::new(0., 0.3, 0., 1.0)
                            } else {
                                Color::new(0.3, 0.3, 0., 1.0)
                            }
                        } else {
                            Color::new(0.3, 0.0, 0., 1.0)
                        },
                        ctx,
                    );
                }
            }
        }
    }
}

// pub const TILE_PERSPECTIVE_HEIGHT: f32 = PIXEL_SIZE * 6.;

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
    // if grid.get_tile(*pos).is_some_and(|tile| !tile.is_passable()) {
    //     // draw "box" around block
    //     // "top side"
    //     rect.h = GRID_CELL_SIZE.0;
    //     rect.y -= TILE_PERSPECTIVE_HEIGHT;
    //     draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
    //     // "front side" facing camera
    //     rect.h = TILE_PERSPECTIVE_HEIGHT;
    //     rect.y += GRID_CELL_SIZE.0;
    //     draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
    // } else {
    draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
    // }
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
            format!("{}: {}", game_ctx.items.get_item(item).unwrap().name, stock).as_str(),
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
