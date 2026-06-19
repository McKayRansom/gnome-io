use macroquad::prelude::rand;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    event::FARM_EVENT_ID,
    game::{GameCtx, Tick, time::Season},
    grid::{Grid, Pos},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SnowManager {
    // wait maybe this should just be a vec...
    pub snow: FxHashMap<Pos, Tick>,
}

// const SNOW_SPAWN_PER_256_TILES: usize = 1;
const SNOW_FALL_TIME: Tick = 50;

const SNOW_BLOCK_ID: BlockId = 103;

impl SnowManager {
    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        if game_ctx.time.season == Season::Winter {
            // spawn some snow
            // for _ in 0..((SNOW_SPAWN_PER_256_TILES * grid.size.x as usize) / 256) {
            let pos: Pos = (rand::gen_range(0, grid.size.x * 8), 0).into();
            if pos.x < grid.size.x {
                self.snow.insert(pos, SNOW_FALL_TIME);
            }
            // }
        }

        // update the snow
        let mut to_remove: Vec<Pos> = Vec::new();
        let mut to_insert: Vec<(Pos, Tick)> = Vec::new();
        for (pos, time) in self.snow.iter_mut() {
            *time = *&time.saturating_sub(1);
            if *time == 0 {
                to_remove.push(*pos);
                // do something now
                match game_ctx.time.season {
                    Season::Winter => {
                        let next_pos = *pos + (0, 1).into();
                        if let Some(tile) = grid.get_tile(next_pos) {
                            if let Some(block) = tile.get_block() {
                                if let Some(block_info) = game_ctx.blocks.get_info(&block) {
                                    if block_info
                                        .mine_event
                                        .is_some_and(|event| event == FARM_EVENT_ID)
                                    {
                                        // remove without dropping...
                                        grid.destroy_block(next_pos, game_ctx);
                                    } else if block != SNOW_BLOCK_ID {
                                        //
                                        grid.place_block(*pos, SNOW_BLOCK_ID, game_ctx);
                                    }
                                }
                            } else {
                                to_insert.push((next_pos, SNOW_FALL_TIME));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        for remove in to_remove.iter() {
            self.snow.remove(remove);
        }
        for insert in to_insert.iter() {
            self.snow.insert(insert.0, insert.1);
        }
    }
}
