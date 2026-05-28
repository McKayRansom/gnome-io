use crate::{
    game::GameCtx,
    grid::{Grid, Pos},
};

use super::{Job, JobManager};

const MINE_TIME: u16 = 60;

pub fn mine(grid: &mut Grid, pos: Pos, game_ctx: &mut GameCtx) -> Option<()> {
    // verify block exists
    let _ = grid.get_tile(pos)?.get_block()?;

    // we could take longer to mine based on block hardness here...

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::mine(pos, MINE_TIME),
    );

    Some(())
}
