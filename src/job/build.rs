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

    // let block_id = game_ctx
    //     .blocks
    //     .get_block(&id)
    //     .unwrap()
    //     .requires
    //     .expect("Can't build with item");

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::new(
            pos,
            BUILD_TIME,
            Some(crate::tile::Entity::Block(id)),
            game_ctx.blocks.get_block(&id).unwrap().requires.clone(),
        ),
    );

    None
}
