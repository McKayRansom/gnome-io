use bitflags::bitflags;
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
bitflags! {
    // Packed pathfinding/state flags kept inline on the tile for cache locality.
    // Add new flags here (has_job, job_type, etc.) as bottlenecks demand.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TileFlags: u8 {
        // the block here cannot be passed through
        const SOLID = 1 << 1;

        // pathfinding can use this tile
        const WALKABLE = 1 << 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "TileRepr")]
pub struct Tile {
    pub contents: Vec<Content>,
    pub biome: TileBiome,
    pub flags: TileFlags,
}

// Deserialize-only shape that accepts both the current `flags` field and the
// legacy `walkable`/`solid` bools, so old save files still load. Each field
// defaults, so a current save (no bools) and a legacy save (no `flags`) both
// deserialize; `From` merges whichever were present. We avoid `Option` here
// because RON requires explicit `Some(..)` for present optional fields.
#[derive(Deserialize)]
struct TileRepr {
    contents: Vec<Content>,
    biome: TileBiome,
    #[serde(default = "TileFlags::empty")]
    flags: TileFlags,
    // legacy fields (pre-TileFlags); absent in current saves.
    #[serde(default)]
    walkable: bool,
    #[serde(default)]
    solid: bool,
}

impl From<TileRepr> for Tile {
    fn from(repr: TileRepr) -> Tile {
        let mut flags = repr.flags;
        // Fold in any legacy bools. No-ops for current saves where both are false.
        flags.set(
            TileFlags::WALKABLE,
            flags.contains(TileFlags::WALKABLE) || repr.walkable,
        );
        flags.set(
            TileFlags::SOLID,
            flags.contains(TileFlags::SOLID) || repr.solid,
        );
        Tile {
            contents: repr.contents,
            biome: repr.biome,
            flags,
        }
    }
}

impl Tile {
    pub fn new(biome: TileBiome) -> Tile {
        Tile {
            contents: Vec::new(),
            biome,
            flags: TileFlags::empty(),
        }
    }

    pub fn new_block(biome: TileBiome, block: BlockId, solid: bool) -> Tile {
        let mut flags = TileFlags::empty();
        flags.set(TileFlags::SOLID, solid);
        Tile {
            contents: vec![Content::Block(block)],
            biome,
            flags,
        }
    }

    pub fn walkable(&self) -> bool {
        self.flags.contains(TileFlags::WALKABLE)
    }

    pub fn solid(&self) -> bool {
        self.flags.contains(TileFlags::SOLID)
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

    pub fn iter_content(&self) -> std::slice::Iter<'_, Content> {
        self.contents.iter()
    }

    pub(crate) fn is_passable(&self) -> bool {
        self.walkable()
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

    pub fn item_count(&self) -> usize {
        self.contents.iter().fold(0, |sum, content| {
            if matches!(content, Content::Item(_)) {
                sum + 1
            } else {
                sum
            }
        })
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn loads_legacy_walkable_solid_save() {
        // pre-TileFlags save format
        let legacy = r#"(
            contents: [],
            biome: Dirt,
            walkable: true,
            solid: false,
        )"#;
        let tile: Tile = ron::from_str(legacy).unwrap();
        assert!(tile.walkable());
        assert!(!tile.solid());
    }

    #[test]
    fn loads_current_flags_save() {
        let current = r#"(
            contents: [],
            biome: Stone,
            flags: ("SOLID"),
        )"#;
        let tile: Tile = ron::from_str(current).unwrap();
        assert!(!tile.walkable());
        assert!(tile.solid());
    }

    #[test]
    fn round_trips_through_flags() {
        let tile = Tile {
            contents: vec![],
            biome: TileBiome::Water,
            flags: TileFlags::WALKABLE | TileFlags::SOLID,
        };
        let s = ron::ser::to_string(&tile).unwrap();
        let back: Tile = ron::from_str(&s).unwrap();
        assert_eq!(back.flags, tile.flags);
    }
}
