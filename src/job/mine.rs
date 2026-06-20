use crate::{
    game::GameCtx,
    grid::{Grid, Pos},
};

use super::Job;

const MINE_TIME: u16 = 60;

pub fn mine(grid: &mut Grid, pos: Pos, game_ctx: &mut GameCtx) -> Option<()> {
    // verify block exists
    if !grid.get_tile(pos)?.has_block() {
        return None;
    }

    // we could take longer to mine based on block hardness here...

    Job::mine(pos, MINE_TIME, super::JobType::MINE).create(grid, game_ctx);

    Some(())
}
