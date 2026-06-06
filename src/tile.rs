use bitflags::{Flags, bitflags};
use serde::{Deserialize, Serialize};

use crate::{
    block::{BlockId, BlockInfoFlags},
    entity::{EntityId, Faction, gnome::GNOME_FACTION},
    event::JobId,
    game::GameCtx,
    item::{ItemId, ItemInfoFlags},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileBiome {
    #[default]
    Sky,
    Dirt,
    Stone,
    Water,
}

pub type ContentItem = (ItemId, ItemInfoFlags);
pub type ContentEntity = (Faction, EntityId);
pub type ContentBlock = (BlockId, BlockInfoFlags);
pub type ContentJob = JobId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Content {
    // ItemId = 0 means match flags
    Item(ContentItem),
    // EntityId = 0 means match faction
    Entity(ContentEntity),
    Block(ContentBlock),
    Job(ContentJob),
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Item(left), Self::Item(right)) => {
                left.0 == right.0
                    || (right.0 == 0 && left.1.contains(right.1))
                    || (left.0 == 0 && right.1.contains(left.1))
            }

            (Self::Entity(left), Self::Entity(right)) => {
                left.1 == right.1 || (right.0 == left.0 && (left.1 == 0 || right.1 == 0))
            }
            (Self::Block(left), Self::Block(right)) => left == right,
            (Self::Job(left), Self::Job(right)) => left == right,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(from = "TileRepr")] // entity_flags: Faction,
#[serde(into = "TileRepr")]
pub struct Tile {
    // we need to manage flags based on content, so make not-pub
    // TODO: make non-pub for correctness...
    pub contents: Vec<Content>,
    pub biome: TileBiome,
    tile_flags: TileFlags,
    block_flags: BlockInfoFlags,
    item_flags: ItemInfoFlags,
}

// We don't want flags to be serialized, they should be modifyable...
#[derive(Serialize, Deserialize)]
struct TileRepr {
    contents: Vec<TileReprContents>,
    biome: TileBiome,
}

#[derive(Serialize, Deserialize)]
enum TileReprContents {
    Item(ItemId),
    Entity(ContentEntity),
    Block(BlockId),
    Job(ContentJob),
}

impl From<TileReprContents> for Content {
    fn from(value: TileReprContents) -> Self {
        match value {
            TileReprContents::Item(item) => Content::Item((item, ItemInfoFlags::default())),
            TileReprContents::Entity(entity) => Content::Entity(entity),
            TileReprContents::Block(block) => Content::Block((block, BlockInfoFlags::default())),
            TileReprContents::Job(job) => Content::Job(job),
        }
    }
}

impl From<Content> for TileReprContents {
    fn from(value: Content) -> Self {
        match value {
            Content::Item((item_id, _)) => TileReprContents::Item(item_id),
            Content::Entity(entity) => TileReprContents::Entity(entity),
            Content::Block((block_id, _)) => TileReprContents::Block(block_id),
            Content::Job(job) => TileReprContents::Job(job),
        }
    }
}

impl From<TileRepr> for Tile {
    fn from(value: TileRepr) -> Self {
        Self {
            contents: value.contents.into_iter().map(Into::into).collect(),
            biome: value.biome,
            ..Default::default()
        }
    }
}

impl From<Tile> for TileRepr {
    fn from(value: Tile) -> Self {
        Self {
            contents: value.contents.into_iter().map(Into::into).collect(),
            biome: value.biome,
        }
    }
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
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TileFlags: u8 {
        // calculated flags
        const WALKABLE = 1 << 0;
        const HAS_ITEMS = 1 << 1;
        const HAS_JOB = 1 << 2;
        const HAS_ENTITY = 1 << 3;
    }
}

impl Tile {
    pub fn new(biome: TileBiome) -> Self {
        Self {
            biome,
            ..Default::default()
        }
    }

    pub fn new_block(biome: TileBiome, block: BlockId, solid: bool) -> Tile {
        let mut flags = BlockInfoFlags::empty();
        flags.set(BlockInfoFlags::SOLID, solid);
        Tile {
            contents: vec![Content::Block((block, flags))],
            biome,
            ..Default::default()
        }
    }

    pub fn walkable(&self) -> bool {
        self.tile_flags.contains(TileFlags::WALKABLE)
    }

    pub fn has_items(&self) -> bool {
        self.tile_flags.contains(TileFlags::HAS_ITEMS)
    }

    pub fn has_job(&self) -> bool {
        self.tile_flags.contains(TileFlags::HAS_JOB)
    }

    pub fn has_entity(&self) -> bool {
        self.tile_flags.contains(TileFlags::HAS_ENTITY)
    }

    pub fn block_flags(&self) -> BlockInfoFlags {
        self.block_flags
    }

    pub fn item_flags(&self) -> ItemInfoFlags {
        self.item_flags
    }

    pub fn remove(&mut self, remove: &Content) -> Option<Content> {
        let result = self
            .contents
            .remove(self.contents.iter().position(|content| content == remove)?);

        self.modified();

        Some(result)
    }

    pub fn add(&mut self, content: Content) {
        self.contents.push(content);

        // NOTE: if this were somehow a bottleneck we could easily only change the flags we need to
        self.modified();
    }

    fn flags_contains(&self, content: &Content) -> bool {
        // see if we can exit early
        match content {
            Content::Item(_) => {
                if !self.has_items() {
                    return false;
                }
            }
            Content::Entity(_) => {
                if !self.has_entity() {
                    return false;
                }
            }
            // Content::Block(_) => if !self.
            Content::Job(_) => {
                if !self.has_job() {
                    return false;
                }
            }
            _ => {}
        }

        true
    }

    pub fn contains(&self, content: &Content) -> bool {
        self.flags_contains(content) && self.contents.contains(content)
    }

    pub fn find(&self, content: &Content) -> Option<Content> {
        if !self.flags_contains(content) {
            return None;
        }
        self.contents.iter().find(|c| *c == content).cloned()
    }

    pub fn iter_content(&self) -> std::slice::Iter<'_, Content> {
        self.contents.iter()
    }

    pub(crate) fn is_passable(&self, faction: Faction) -> bool {
        self.walkable() 
            && self.get_entity().is_none_or(|entity| entity.0 == faction)
            // NOTE: Only block non-gnomes so we don't have to store the faction of blocks
            // will need to be changed if we ever have multiplayer
            && (faction == GNOME_FACTION || !self.block_flags.contains(BlockInfoFlags::DOOR))
    }

    pub fn get_block(&self) -> Option<BlockId> {
        for content in self.contents.iter() {
            if let Content::Block(block) = *content {
                return Some(block.0);
            }
        }
        None
    }

    pub(crate) fn get_job(&self) -> Option<JobId> {
        if !self.has_job() {
            return None;
        }
        for content in &self.contents {
            if let Content::Job(id) = content {
                return Some(*id);
            }
        }
        return None;
    }

    pub(crate) fn get_entity(&self) -> Option<ContentEntity> {
        if !self.has_entity() {
            return None;
        }
        for content in &self.contents {
            if let Content::Entity(entity) = content {
                return Some(*entity);
            }
        }
        None
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

    // fixup our block flags, our walkable flag needs to be set by grid based on adjacent tiles
    pub(crate) fn modified(&mut self) {
        // update our flags
        // do not clear walakable
        self.tile_flags = self.tile_flags.intersection(TileFlags::WALKABLE);
        self.block_flags.clear();
        self.item_flags.clear();

        for content in self.contents.iter() {
            match content {
                Content::Item((_id, flags)) => {
                    self.tile_flags.insert(TileFlags::HAS_ITEMS);
                    self.item_flags.insert(*flags)
                }
                Content::Entity(_) => self.tile_flags.insert(TileFlags::HAS_ENTITY),
                Content::Block((_id, flags)) => self.block_flags.insert(*flags),
                Content::Job(_) => self.tile_flags.insert(TileFlags::HAS_JOB),
            }
        }
    }

    // This is for fixing after loading a save
    // We want to be able to change block_info and this will migrate it
    // and this way we store as little as possible in the save
    pub fn fixup(&mut self, game_ctx: &GameCtx) {
        for content in self.contents.iter_mut() {
            match content {
                Content::Item(item) => {
                    *item = game_ctx
                        .items
                        .get_content(&item.0)
                        .expect("Unknown item in map")
                }
                Content::Entity(_) => {}
                Content::Block(block) => {
                    *block = game_ctx
                        .blocks
                        .get_content(&block.0)
                        .expect("Unknown block in map")
                }
                Content::Job(_) => {}
            }
        }
    }

    pub(crate) fn set_walkable(&mut self, walkable: bool) {
        self.tile_flags.set(TileFlags::WALKABLE, walkable);
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn loads_legacy_walkable_solid_save() {
        // pre-TileFlags save format
        // let legacy = r#"(
        //     contents: [],
        //     biome: Dirt,
        //     walkable: true,
        //     solid: false,
        // )"#;
        // let tile: Tile = ron::from_str(legacy).unwrap();
        // assert!(tile.walkable());
        // assert!(!tile.block_flags.contains(BlockInfoFlags::SOLID));
    }

    #[test]
    fn content_item_eq() {
        assert_eq!(
            Content::Item((123, ItemInfoFlags::FOOD)),
            Content::Item((123, ItemInfoFlags::FOOD))
        );
        assert_eq!(
            Content::Item((0, ItemInfoFlags::FOOD)),
            Content::Item((123, ItemInfoFlags::FOOD))
        );
        assert_eq!(
            Content::Item((123, ItemInfoFlags::FOOD)),
            Content::Item((0, ItemInfoFlags::FOOD))
        )
    }

    #[test]
    fn content_entity_eq() {
        // assert_ne!(Content::Entity((1, 2)), Content::Entity((2, 2)));
        assert_eq!(Content::Entity((2, 2)), Content::Entity((2, 2)),);
        assert_eq!(Content::Entity((2, 0)), Content::Entity((2, 2)),);
        assert_eq!(Content::Entity((2, 2)), Content::Entity((0, 2)),)
    }

    #[test]
    fn tile_contains_item() {
        let mut tile = Tile::default();
        tile.add(Content::Item((123, ItemInfoFlags::FOOD)));

        assert!(tile.has_items());
        assert!(!tile.has_entity());
        assert!(!tile.has_job());

        assert!(tile.contains(&Content::Item((123, ItemInfoFlags::FOOD))));
        assert!(tile.contains(&Content::Item((0, ItemInfoFlags::FOOD))));
        assert!(!tile.contains(&Content::Item((1, ItemInfoFlags::FOOD))));
    }

    // #[test]
    // fn loads_current_flags_save() {
    //     let current = r#"(
    //         contents: [],
    //         biome: Stone,
    //         flags: ("SOLID"),
    //     )"#;
    //     let tile: Tile = ron::from_str(current).unwrap();
    //     assert!(!tile.walkable());
    //     // assert!(tile.solid());
    // }

    // #[test]
    // fn round_trips_through_flags() {
    //     let tile = Tile {
    //         contents: vec![],
    //         biome: TileBiome::Water,
    //         flags: TileFlags::WALKABLE | TileFlags::SOLID,
    //     };
    //     let s = ron::ser::to_string(&tile).unwrap();
    //     let back: Tile = ron::from_str(&s).unwrap();
    //     assert_eq!(back.flags, tile.flags);
    // }
}
