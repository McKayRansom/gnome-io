// use hecs::Entity;

use crate::gnome::GnomeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileBiome {
    Dirt,
    Stone,
    Water,
}

#[derive(Debug, Clone)]
pub struct Tile {
    // pub item: Option<Entity>,
    pub gnome: Option<GnomeId>,
    pub is_passable: bool,
    pub biome: TileBiome,
}

impl Tile {
    pub fn new(biome: TileBiome) -> Tile {
        Tile {
            // item: None,
            gnome: None,
            is_passable: true,
            biome,
        }
    }

    pub fn new_block(biome: TileBiome) -> Tile {
        Tile {
            // item: None,
            gnome: None,
            is_passable: false,
            biome,
        }
    }

    // pub fn set_item(&mut self, item: Entity) {
    //     self.item = Some(item);
    // }

    // pub fn set_gnome(&mut self, gnome: Entity) {
    //     self.gnome = Some(gnome);
    // }

    pub fn set_passable(&mut self, passable: bool) {
        self.is_passable = passable;
    }
    
    pub(crate) fn is_passable(&self) -> bool {
        self.is_passable
    }
}
