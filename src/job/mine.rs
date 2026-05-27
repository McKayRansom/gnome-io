use crate::{
    block::blocks,
    game::GameCtx,
    grid::{Grid, Pos},
    tile::Content,
};

use super::{Job, JobManager};

const MINE_TIME: u16 = 60;

pub fn mine(grid: &mut Grid, pos: Pos, game_ctx: &mut GameCtx) -> Option<()> {
    let _ = grid.get_tile(pos)?.get_block()?;

    // self.spawn_job(Job::new(dig_pos?, pos));
    // self.tiles_queued.push_back(pos);
    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::new(pos, MINE_TIME, Some(Content::Block(blocks::NONE)), vec![]),
    );

    Some(())
}
