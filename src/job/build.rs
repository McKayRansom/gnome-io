use crate::{
    game::{GameCtx, Tick},
    grid::{Grid, Pos},
};

use super::{Job, JobManager};

const BUILD_TIME: Tick = 30;

pub fn build(grid: &mut Grid, pos: Pos, block_name: &str, game_ctx: &mut GameCtx) -> Option<()> {
    let id = game_ctx
        .blocks
        .get_id(block_name)
        .expect("Bad block_name passed to build()");

    if grid
        .get_tile(pos)?
        .get_block()
        .is_some_and(|block| block == id)
    {
        return None;
    }

    let block_info = game_ctx.blocks.get_info(&id).expect("Can't build block");

    let requires = block_info
        .requires
        .iter()
        .map(|item_id| (*item_id, game_ctx.items.get_info(item_id).unwrap().flags))
        .collect();

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::build(pos, BUILD_TIME, (id, block_info.flags), requires, super::JobType::BUILD),
    );

    None
}
