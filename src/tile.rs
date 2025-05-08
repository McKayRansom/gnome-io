// use hecs::Entity;

use crate::{block::BlockId, gnome::GnomeId, item::ItemId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileBiome {
    Dirt,
    Stone,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entity {
    Item(ItemId),
    Gnome(GnomeId),
    Block(BlockId),
}

#[derive(Debug, Clone)]
pub struct Tile {
    // should this be hashmap?
    entities: Vec<Entity>,
    pub biome: TileBiome,
    // TODO: PathfindingInfo{}
    pub walkable: bool,
}

impl Tile {
    pub fn new(biome: TileBiome) -> Tile {
        Tile {
            entities: Vec::new(),
            biome,
            walkable: biome != TileBiome::Water,
        }
    }

    pub fn new_block(biome: TileBiome, block: BlockId) -> Tile {
        Tile {
            entities: vec![Entity::Block(block)],
            biome,
            walkable: false,
        }
    }

    pub fn get_block(&self) -> Option<BlockId> {
        for entity in self.entities.iter() {
            if let Entity::Block(block_id) = *entity {
                return Some(block_id);
            }
        }
        None
    }

    pub fn remove_entity(&mut self, remove: &Entity) -> Option<Entity> {
        Some(
            self.entities
                .remove(self.entities.iter().position(|entity| entity == remove)?),
        )
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn contains(&self, entity: &Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn iter_entities(&self) -> std::slice::Iter<'_, Entity> {
        self.entities.iter()
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
