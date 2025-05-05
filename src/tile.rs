// use hecs::Entity;

use crate::{block::BlockId, gnome::GnomeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileBiome {
    Dirt,
    Stone,
    Water,
}

#[derive(Debug, Clone)]
pub struct Tile {
    // pub item: Option<Entity>, // might need to be a vec sadly
    pub gnome: Option<GnomeId>,
    pub biome: TileBiome,
    pub block: Option<BlockId>, // only 1 block allowed per tile
}

impl Tile {
    pub fn new(biome: TileBiome) -> Tile {
        Tile {
            // item: None,
            gnome: None,
            biome,
            block: None,
        }
    }

    pub fn new_block(biome: TileBiome, block: BlockId) -> Tile {
        Tile {
            // item: None,
            gnome: None,
            biome,
            block: Some(block),
        }
    }

    // pub fn set_item(&mut self, item: Entity) {
    //     self.item = Some(item);
    // }

    // pub fn set_gnome(&mut self, gnome: Entity) {
    //     self.gnome = Some(gnome);
    // }

    // pub fn set_passable(&mut self, passable: bool) {
    //     self.is_passable = passable;
    // }
    
    pub(crate) fn is_passable(&self) -> bool {
        self.block.is_none()
    }
}
