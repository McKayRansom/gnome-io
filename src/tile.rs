// use hecs::Entity;

use crate::{block::BlockId, gnome::GnomeId, item::ItemId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileBiome {
    Dirt,
    Stone,
    Water,
}

#[derive(Debug, Clone)]
pub struct Tile {
    // should this be hashmap?
    pub items: Vec<ItemId>,
    pub gnome: Option<GnomeId>,
    pub biome: TileBiome,
    pub block: Option<BlockId>, // only 1 block allowed per tile
    // TODO: PathfindingInfo{}
    pub walkable: bool,
}

impl Tile {
    pub fn new(biome: TileBiome) -> Tile {
        Tile {
            items: Vec::new(),
            gnome: None,
            biome,
            block: None,
            walkable: biome != TileBiome::Water
        }
    }

    pub fn new_block(biome: TileBiome, block: BlockId) -> Tile {
        Tile {
            items: Vec::new(),
            gnome: None,
            biome,
            block: Some(block),
            walkable: false,
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
        self.walkable
    }
}
