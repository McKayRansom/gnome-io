use crate::{
    block::BlockId,
    game::{GameCtx, Tick},
    grid::{Grid, Pos},
    item::ItemId,
};

use super::{Job, JobManager};

const BUILD_TIME: Tick = 30;

pub fn build(grid: &Grid, pos: Pos, id: ItemId, game_ctx: &mut GameCtx) -> Option<()> {
    let block = grid.get_tile(pos)?.block;
    if block.is_some() {
        return None;
    }

    let block_id = game_ctx
        .items
        .get_item(&id)
        .unwrap()
        .builds
        .expect("Can't build with item");

    JobManager::create_job(
        &mut game_ctx.events,
        Job::new(pos, BUILD_TIME, Some(block_id), vec![id]),
    );

    None
}
