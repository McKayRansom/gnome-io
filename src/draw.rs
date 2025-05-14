// use hecs::World;
use macroquad::{
    color::{Color, colors},
    math::{Rect, vec2},
    shapes::draw_rectangle_lines,
};

use crate::{
    context::Context,
    game::{Game, GameCtx, Gnomes},
    gnome::{self, SLEEP_TIRED},
    grid::{
        Grid, Pos,
        pos::{PIXEL_SIZE, dirs},
    },
    text::draw_text,
    tile::{Entity, TileBiome}, // tileset::{GRID_CELL_SIZE, PIXEL_SIZE, Sprite, pos_to_rect, sprites},
};

// pub const SPRITES: &[&[&str]] = &[&["", "", "", "", "", "", "bed", ""],
// &["gnome", "grass", ]];

pub mod sprites {
    use quad_lib::tileset::Sprite;

    pub const GNOME: Sprite = Sprite::new(3, 0);

    pub const GRASS: Sprite = Sprite::new(3, 1);
    pub const STONE: Sprite = Sprite::new(7, 5);
    pub const STONE_FLOOR: Sprite = Sprite::new(1, 2);
    pub const _DIRT: Sprite = Sprite::new(1, 1);
    pub const WATER: Sprite = Sprite::new(3, 2);

    pub const ORE: Sprite = Sprite::new(7, 6);
    pub const STONE_ITEM: Sprite = Sprite::new(5, 2);

    pub const TREE: Sprite = Sprite::new(7, 4);
    pub const WOOD: Sprite = Sprite::new(5, 5);

    pub const FURNACE: Sprite = Sprite::new(1, 5);
    pub const CRAFT_TABLE: Sprite = Sprite::new(1, 6);
    pub const _CHEST: Sprite = Sprite::new(1, 7);
    pub const BED: Sprite = Sprite::new(0, 6);

    pub const BREAD: Sprite = Sprite::new(7, 7);

    pub const UNKNOWN_ITEM: Sprite = Sprite::new(7, 0);
    pub const THINK: Sprite = Sprite::new(6, 0);
    pub const SLEEP: Sprite = Sprite::new(6, 1);

    pub const WHEAT_SEED: Sprite = Sprite::new(5, 3);
    pub const WHEAT_GRAIN: Sprite = Sprite::new(5, 7);

    pub const WHEAT_0: Sprite = Sprite::new(3, 3);
    pub const WHEAT_1: Sprite = Sprite::new(3, 4);
    pub const WHEAT_2: Sprite = Sprite::new(3, 5);
    pub const WHEAT_3: Sprite = Sprite::new(3, 6);
    pub const WHEAT_4: Sprite = Sprite::new(3, 7);

    pub const PLAY: Sprite = Sprite::new(0, 0);
    pub const PAUSE: Sprite = Sprite::new(0, 0);
    pub const FAST_FORWARD: Sprite = Sprite::new(0, 0);
    pub const MENU: Sprite = Sprite::new(0, 0);
}

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
            if tile.biome == TileBiome::Stone
                && !dirs::ALL.iter().any(|dir| {
                    grid.get_tile(pos + *dir)
                        .is_some_and(|tile| tile.is_passable())
                })
            {
                continue;
            }
            let dest: Rect = ctx.camera.to_screen_rect(pos.into());
            ctx.tileset.draw_tile(
                match tile.biome {
                    TileBiome::Dirt => sprites::GRASS,
                    TileBiome::Stone => sprites::STONE_FLOOR,
                    TileBiome::Water => sprites::WATER,
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
                ctx.tileset.draw_tile(block.sprite, &dest, colors::WHITE);
            }
            // then draw items
            for item in tile.iter_entities() {
                if let Entity::Item(item) = item {
                    ctx.tileset.draw_tile(
                        if let Some(item) = game_ctx.items.get_item(item) {
                            item.sprite
                        } else {
                            sprites::UNKNOWN_ITEM
                        },
                        &dest,
                        colors::WHITE,
                    );
                }
            }

            // then gnomes
            for item in tile.iter_entities() {
                if let Entity::Gnome(gnome) = item {
                    // ctx.tileset.draw_tile(sprites, dest, color);
                    let gnome = gnomes.get(gnome).unwrap();
                    let think_box: Rect = ctx.camera.to_screen_rect((pos + dirs::UP).into());
                    if gnome.tired < SLEEP_TIRED {
                        ctx.tileset
                            .draw_tile(sprites::THINK, &think_box, colors::WHITE);
                        ctx.tileset
                            .draw_tile(sprites::BED, &think_box, colors::WHITE);
                    } else if gnome.sleeping {
                        ctx.tileset
                            .draw_tile(sprites::THINK, &think_box, colors::WHITE);
                        ctx.tileset
                            .draw_tile(sprites::SLEEP, &think_box, colors::WHITE);
                    } else if gnome.food < gnome::FOOD_EAT {
                        ctx.tileset
                            .draw_tile(sprites::THINK, &think_box, colors::WHITE);
                        ctx.tileset
                            .draw_tile(sprites::BREAD, &think_box, colors::WHITE);
                    }
                    ctx.tileset.draw_tile(sprites::GNOME, &dest, colors::WHITE);
                }
            }

            // draw jobs last (on top)
            for item in tile.iter_entities() {
                if let Entity::Job(job) = item {
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
    draw_text(
        ctx,
        format!("Day {} of {:?} Year {}", time.day, time.season, time.year).as_str(),
        ctx.screen_size.x - 100.,
        20.,
        crate::text::Size::Medium,
        colors::WHITE,
    );
}
