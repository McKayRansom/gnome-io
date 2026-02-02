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
pub enum Content {
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
        return None
    }
}
