use hecs::World;
use macroquad::{
    color::{Color, colors},
    math::Rect,
};

use crate::{
    context::Context,
    game::Game,
    gnome::Gnome,
    grid::{Grid, Pos}, job::Job,
};

pub fn draw_game(game: &Game, ctx: &Context) {
    draw_tiles(&game.grid, ctx);
    draw_gnomes(&game.world, ctx);
    draw_jobs(&game.jobs, ctx);
}

fn draw_tiles(grid: &Grid, ctx: &Context) {
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            ctx.tileset.draw_rect(
                &pos_to_rect((x, y).into()),
                if grid
                    .get_tile((x, y).into())
                    .is_some_and(|tile| tile.is_passable())
                {
                    colors::BROWN
                } else {
                    colors::GREEN
                },
            );
        }
    }
}

fn draw_gnomes(world: &World, ctx: &Context) {
    for (_entity, (gnome,)) in world.query::<(&mut Gnome,)>().iter() {
        // gnome.update(&mut self.grid, &mut self.jobs);
        ctx.tileset
            .draw_rect(&pos_to_rect(gnome.pos), Color::new(0., 0., 1., 0.5));
    }
}

fn draw_jobs(jobs: &Vec<Job>, ctx: &Context) {
    for job in jobs.iter() {
        ctx.tileset
            .draw_rect(&pos_to_rect(job.pos), Color::new(1., 0., 0., 0.5));
    }
}

// Default zoom pixel size of Position
pub const GRID_CELL_SIZE: (f32, f32) = (64., 64.);
pub const PIXEL_SIZE: f32 = 64. / 16.;

pub fn pos_to_rect(pos: Pos) -> Rect {
    Rect::new(
        pos.x as f32 * GRID_CELL_SIZE.0,
        pos.y as f32 * GRID_CELL_SIZE.1, /* - (pos.z as f32 * GRID_Z_OFFSET) */
        GRID_CELL_SIZE.0,
        GRID_CELL_SIZE.1,
    )
}
