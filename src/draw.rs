// use hecs::World;
use macroquad::{
    color::{Color, colors},
    math::Rect,
    shapes::draw_rectangle_lines,
};
use quad_lib::tileset::Sprite;

use crate::{
    context::Context,
    game::{Game, GameCtx},
    grid::{
        Grid, Pos,
        pos::{GRID_CELL_SIZE, PIXEL_SIZE},
    },
    tile::{Entity, TileBiome},
    // tileset::{GRID_CELL_SIZE, PIXEL_SIZE, Sprite, pos_to_rect, sprites},
};

pub mod sprites {
    use quad_lib::tileset::Sprite;

    pub const GNOME: Sprite = Sprite::new(0, 0);

    pub const STONE: Sprite = Sprite::new(0, 3);
    pub const ORE: Sprite = Sprite::new(0, 4);
    pub const STONE_ITEM: Sprite = Sprite::new(2, 2);

    pub const TREE: Sprite = Sprite::new(2, 4);
    pub const WOOD: Sprite = Sprite::new(2, 5);

    pub const FURNACE: Sprite = Sprite::new(0, 5);
    pub const CRAFT_TABLE: Sprite = Sprite::new(0, 6);
    pub const _CHEST: Sprite = Sprite::new(0, 7);

    pub const BREAD: Sprite = Sprite::new(3, 7);

    pub const UNKOWN_ITEM: Sprite = Sprite::new(3, 0);
}

pub fn draw_game(game: &Game, ctx: &Context) {
    draw_tiles(&game.grid, &game.game_ctx, ctx);
    // draw_gnomes(&game.gnomes, ctx);
}

fn draw_tiles(grid: &Grid, game_ctx: &GameCtx, ctx: &Context) {
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let pos: Pos = (x, y).into();
            let tile = grid.get_tile(pos).unwrap();
            let dest: Rect = pos.into();
            // manual adjustment because tiles are weirdly shaped...
            let dest: Rect = ctx.camera.to_screen_rect(Rect {
                x: dest.x,
                y: dest.y - dest.h,
                w: dest.w,
                h: dest.h * 2.,
            });
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
                        }
                        Entity::Job(job) => {
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

pub fn draw_tile_outline(grid: &Grid, pos: &Pos, color: Color, ctx: &Context) {
    let mut rect: Rect = (*pos).into();
    if grid.get_tile(*pos).is_some_and(|tile| !tile.is_passable()) {
        // draw "box" around block
        // "top side"
        rect.h = GRID_CELL_SIZE.0;
        rect.y -= TILE_PERSPECTIVE_HEIGHT;
        draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
        // "front side" facing camera
        rect.h = TILE_PERSPECTIVE_HEIGHT;
        rect.y += GRID_CELL_SIZE.0;
        draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
    } else {
        draw_rect_outline(&ctx.camera.to_screen_rect(rect), color, ctx);
    }
}
