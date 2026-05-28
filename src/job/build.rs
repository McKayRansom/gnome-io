use crate::{
    block::BlockId,
    game::{GameCtx, Tick},
    grid::{Grid, Pos},
};

use super::{Job, JobManager};

const BUILD_TIME: Tick = 30;

pub fn build(grid: &mut Grid, pos: Pos, id: BlockId, game_ctx: &mut GameCtx) -> Option<()> {
    if grid.get_tile(pos)?.get_block().is_some() {
        return None;
    }

    let block_info = game_ctx.blocks.get_block(&id).expect("Can't build block");

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::build(pos, BUILD_TIME, id, block_info.requires.clone())
    );

    None
}
