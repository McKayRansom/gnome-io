use crate::{
    block::BlockInfoFlags,
    event::{BUILD_EVENT_ID, Event, EventTypes, Events},
    game::{GameCtx, Tick},
    grid::{Grid, Pos, pos::dirs},
    job::{JobState, JobType, Step},
};

use super::Job;

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

    // let is_unsafe = pos_is_unsafe_to_build(pos, grid, &game_ctx.events);

    let mut new_job = Job::build(
        pos,
        BUILD_TIME,
        (id, block_info.flags),
        requires,
        super::JobType::BUILD,
    );
    new_job.steps.push(Step::PushEvent(Event {
        id: BUILD_EVENT_ID,
        pos,
        value: EventTypes::BlockUpdateEvent(0, id),
    }));
    let _id = new_job.clone().create(grid, &mut game_ctx.events);
    // TODO: This didnt' work
    // JANKERCITY WARNING!!! WEEWOOWEEWOO
    // new_job.id = id;
    // safety check: Could this job "trap" another build job
    // if is_unsafe {
    //     new_job.state = super::JobState::BuildQueued;
    //     game_ctx.events.update_job(&new_job);
    // }

    None
}

fn pos_is_unsafe_to_trap(pos: Pos, grid: &Grid, events: &Events) -> bool {
    grid.is_valid_pos(pos)
        && dirs::ALL.iter().all(|dir| {
            grid.get_tile(pos + *dir).is_none_or(|tile| {
                tile.block_flags().contains(BlockInfoFlags::SOLID)
                    || tile.get_job().is_some_and(|job| {
                        events.job_get(&job).is_some_and(|job| {
                            job.category == JobType::BUILD && job.state != JobState::BuildQueued
                        })
                    })
            })
        })
}

fn pos_is_unsafe_to_build(pos: Pos, grid: &Grid, events: &Events) -> bool {
    dirs::ALL
        .iter()
        .any(|dir| pos_is_unsafe_to_trap(pos + *dir, grid, events))
}

pub fn update(grid: &mut Grid, game_ctx: &mut GameCtx) {
    while let Some(event) = game_ctx.events.pop_event(BUILD_EVENT_ID) {
        match event.value {
            EventTypes::BlockUpdateEvent(_, _) => {
                // queue any adjacent stalled events
                for dir in dirs::ALL {
                    let Some(Some(Some(mut job))) = grid.get_tile(event.pos + dir).map(|tile| {
                        tile.get_job()
                            .map(|job| game_ctx.events.job_get(&job).cloned())
                    }) else {
                        continue;
                    };

                    if job.category == JobType::BUILD
                        && job.state == JobState::BuildQueued
                        && !pos_is_unsafe_to_build(event.pos + dir, grid, &game_ctx.events)
                    {
                        job.state = JobState::Ready;
                        game_ctx.events.update_job(&job);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{block::BlockInfo, tile::Content};

    use super::*;

    #[test]
    fn test_unsafe_to_trap() {
        let mut grid = Grid::new((16, 16).into());
        let mut events = Events::default();

        assert!(!pos_is_unsafe_to_trap((0, 0).into(), &grid, &events));

        let job_id = Job::build(
            (1, 0).into(),
            0,
            (123, BlockInfoFlags::default()),
            vec![],
            JobType::BUILD,
        )
        .create(&mut grid, &mut events);

        grid.create((1, 0).into(), Content::Job(job_id), &mut events);

        assert!(!pos_is_unsafe_to_trap((0, 0).into(), &grid, &events));

        grid.create(
            (0, 1).into(),
            Content::Block((123, BlockInfoFlags::SOLID)),
            &mut events,
        );

        assert!(pos_is_unsafe_to_trap((0, 0).into(), &grid, &events));
        assert!(pos_is_unsafe_to_build((1, 0).into(), &grid, &events));
        assert!(pos_is_unsafe_to_build((0, 1).into(), &grid, &events));
    }

    #[test]
    fn test_build_unsafe() {
        let mut grid = Grid::new((16, 16).into());
        let mut game_ctx = GameCtx::default();
        game_ctx.blocks.add_block(123, BlockInfo::default());
        game_ctx.blocks.add_name("foo".into(), 123);

        grid.create(
            (0, 1).into(),
            Content::Block((123, BlockInfoFlags::SOLID)),
            &mut game_ctx.events,
        );

        build(&mut grid, (0, 0).into(), "foo", &mut game_ctx);
        let job = game_ctx.events.job_get(&1).unwrap();
        assert_eq!(job.state, JobState::Ready);

        assert_eq!(
            pos_is_unsafe_to_build((1, 0).into(), &grid, &game_ctx.events),
            false,
        );

        build(&mut grid, (1, 0).into(), "foo", &mut game_ctx);
        let job = game_ctx.events.job_get(&2).unwrap();
        assert_eq!(job.state, JobState::BuildQueued);
    }
}
