
use crate::{
    event::{Event, EventManager}, game::GameCtx, grid::{Grid, Pos}
};

use super::{JOB_QUEUE, Job};

pub struct MineJob {
    pos: Pos,
    time: u16,
}

const MINE_TIME: u16 = 60;

pub struct MineManager {
    // pub tiles_queued: VecDeque<Pos>,
    // pub tiles_in_progress: Vec<Pos>,
}

impl MineManager {
    pub fn new() -> Self {
        Self {
            // tiles_queued: VecDeque::new(),
            // tiles_in_progress: Vec::new(),
        }
    }

    pub fn mine(&mut self, grid: &Grid, pos: Pos, game_ctx: &mut GameCtx) -> Option<()> {
        let _ = grid.get_tile(pos)?.block?;

        // self.spawn_job(Job::new(dig_pos?, pos));
        // self.tiles_queued.push_back(pos);
        game_ctx.events.push_event(Event {
            id: JOB_QUEUE,
            value: Box::new(Job::new(pos, MINE_TIME, None)),
        });

        Some(())
    }

    // pub fn update(&mut self, )

    // pub fn finished(&mut self, pos: Pos) {
    //     if let Some(index) = self.tiles_in_progress.iter().position(|p| p == &pos) {
    //         self.tiles_in_progress.remove(index);
    //     }
    // }

    // pub fn failed(&mut self, pos: Pos) {
    //     log::warn!("Mine job failed at pos: {:?}", pos);
    //     if let Some(index) = self.tiles_in_progress.iter().position(|p| p == &pos) {
    //         self.tiles_in_progress.remove(index);
    //         // how do we avoid infinitely adding back to queue?
    //     }
    // }
}

// pub fn
