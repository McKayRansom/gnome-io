// use hecs::Entity;

use serde::{Deserialize, Serialize};

use crate::{block::BlockId, event::JobId, gnome::GnomeId, item::ItemId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum TileBiome {
    Dirt,
    Stone,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum Entity {
    Item(ItemId),
    Gnome(GnomeId),
    Block(BlockId),
    Job(JobId),
}

/*
 * Theory of pathfinding otimization:
 * - Games like transport-io, gnomoria have shown that pathfinding is often the bottleneck.
 * - For optimal pathfinding, all nescesary information to make a path should be in a spacially oriented datastructure (Grid or chunked grid in the future)
 * - This allows all pathfinding lookups to have cache hits on not have to I.E. dereference other vectors or look up in hashmaps (poor cache locality)
 * - So our tile needs to store pathfinding information here in the struct and other info can be stored elsewhere
 */
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Tile {
    // should this be hashmap?
    pub entities: Vec<Entity>,
    pub biome: TileBiome,
    // TODO: PathfindingInfo{}
    pub walkable: bool,
    // TODO: TileFlags (walkable, biome, has_job, job_type, etc...) for whatever the bottlenecks are so we don't always have to look through entities[]
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
