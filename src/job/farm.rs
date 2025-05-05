use crate::{
    block::{BlockId, BlockType, Blocks},
    game::Tick,
    grid::{Grid, Pos},
    tile::TileBiome,
    tileset::Sprite,
};

use super::{BuildJob, Job};

// pub struct Farm {
//     pub start: Pos,
//     pub end: Pos,
// }

const FARM_GROWTH_STAGES: usize = 5;

pub struct FarmManager {
    farm_blocks: [BlockId; FARM_GROWTH_STAGES],
    farm_pos: Vec<Pos>,
}

impl FarmManager {
    pub fn new(blocks: &mut Blocks) -> Self {
        FarmManager {
            farm_blocks: [
                blocks.add_block(BlockType::new(Sprite::new(1, 3))),
                blocks.add_block(BlockType::new(Sprite::new(1, 4))),
                blocks.add_block(BlockType::new(Sprite::new(1, 5))),
                blocks.add_block(BlockType::new(Sprite::new(1, 6))),
                blocks.add_block(BlockType::new(Sprite::new(1, 7))),
            ],
            farm_pos: Vec::new(),
        }
    }

    pub fn new_farm(&mut self, grid: &Grid, pos: Pos) -> Option<()> {
        if grid.get_tile(pos)?.biome != TileBiome::Dirt {
            return None;
        }

        if !self.farm_pos.contains(&pos) {
            self.farm_pos.push(pos);
        }
        Some(())
    }

    const TILL_TIME: Tick = 20;
    const HARVEST_TIME: Tick = 20;

    // this sounds very expensive...
    pub fn find_job(&mut self, grid: &Grid) -> Option<Box<dyn Job>> {
        // tilling...
        // assumes Ids are in order...
        let tilled_id = self.farm_blocks.first().unwrap();
        let grown_id = self.farm_blocks.last().unwrap();
        for pos in &self.farm_pos {
            if let Some(tile) = grid.get_tile(*pos) {
                if tile
                    .block
                    .is_none_or(|block| &block < tilled_id || &block > grown_id)
                {
                    // till
                    return Some(BuildJob::new(*pos, Self::TILL_TIME, *tilled_id))
                } else if tile.block.is_some_and(|block| &block == grown_id) {
                    // harvest
                    return Some(BuildJob::new(*pos, Self::HARVEST_TIME, *grown_id))
                }
            }
        }
        None
    }
}
