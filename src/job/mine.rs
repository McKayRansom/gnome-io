use crate::{
    event::Event,
    game::GameCtx,
    grid::{Grid, Pos},
};

use super::{JOB_QUEUE, Job};

const MINE_TIME: u16 = 60;

pub fn mine(grid: &Grid, pos: Pos, game_ctx: &mut GameCtx) -> Option<()> {
    let _ = grid.get_tile(pos)?.block?;

    // self.spawn_job(Job::new(dig_pos?, pos));
    // self.tiles_queued.push_back(pos);
    game_ctx.events.push_event(Event {
        id: JOB_QUEUE,
        value: Box::new(Job::new(pos, MINE_TIME, None, vec![])),
    });

    Some(())
}
