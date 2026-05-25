use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    entity::{EntityId, Faction},
    event::JobId,
    item::ItemId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileBiome {
    Sky,
    Dirt,
    Stone,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Content {
    Item(ItemId),
    Entity((Faction, EntityId)),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    // should this be hashmap?
    pub contents: Vec<Content>,
    pub biome: TileBiome,
    // TODO: PathfindingInfo{}
    pub walkable: bool,
    // TODO: TileFlags (walkable, biome, has_job, job_type, etc...) for whatever the bottlenecks are so we don't always have to look through entities[]
}

impl Tile {
    pub fn new(biome: TileBiome) -> Tile {
        Tile {
            contents: Vec::new(),
            biome,
            walkable: biome != TileBiome::Water,
        }
    }

    pub fn new_block(biome: TileBiome, block: BlockId) -> Tile {
        Tile {
            contents: vec![Content::Block(block)],
            biome,
            walkable: false,
        }
    }

    pub fn get_block(&self) -> Option<BlockId> {
        for content in self.contents.iter() {
            if let Content::Block(block_id) = *content {
                return Some(block_id);
            }
        }
        None
    }

    pub fn remove(&mut self, remove: &Content) -> Option<Content> {
        Some(
            self.contents
                .remove(self.contents.iter().position(|content| content == remove)?),
        )
    }

    pub fn add(&mut self, content: Content) {
        self.contents.push(content);
    }

    pub fn contains(&self, content: &Content) -> bool {
        if let Content::Entity((faction, id)) = content {
            if *id == 0 {
                for content in self.contents.iter() {
                    if let Content::Entity((faction_2, _id_2)) = content {
                        if faction == faction_2 {
                            return true;
                        }
                    }
                }
                return false;
            }
        }
        self.contents.contains(content)
    }

    pub fn iter_entities(&self) -> std::slice::Iter<'_, Content> {
        self.contents.iter()
    }

    pub(crate) fn is_passable(&self) -> bool {
        self.walkable
    }

    pub(crate) fn get_job(&self) -> Option<JobId> {
        for content in &self.contents {
            if let Content::Job(id) = content {
                return Some(*id);
            }
        }
        return None;
    }

    pub(crate) fn get_entity(&self, entity: (u8, u32)) -> u32 {
        for content in &self.contents {
            if let Content::Entity((faction, id)) = content {
                if faction == &entity.0 && (entity.1 == 0 || &entity.1 == id) {
                    return *id;
                }
            }
        }
        0
    }
}
