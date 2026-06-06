use macroquad::file::load_file;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BLOCK_NONE, BlockInfoFlags},
    event::{EVENT_NONE, EventNames},
    game::{
        Tick,
        time::{HOURS_PER_DAY, TICKS_PER_HOUR},
    },
    item::{ITEM_NONE, Items},
    tile::ContentBlock,
};

use super::{BlockId, BlockInfo};

type BlockInfos = FxHashMap<BlockId, BlockInfo>;
type BlockIds = FxHashMap<String, BlockId>;

#[derive(Default, Serialize, Deserialize)]
pub struct Blocks {
    #[serde(skip_deserializing, skip_serializing)]
    infos: BlockInfos,
    // saved only so we can check for changes and/or future migrations if required...
    ids: BlockIds,
}

// Deserialize-only struct
#[derive(Debug, Clone, Deserialize)]
struct BlocksSave {
    blocks: FxHashMap<String, BlockInfoSave>,
}

#[derive(Debug, Clone, Deserialize)]
struct GrowthSave {
    // To keep tick math out of blocks.ron this is in days for now...
    days: Tick,
    // String to avoid order mattering in blocks.ron, could also make a 2nd pass but that's annoying...
    into: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BlockInfoSave {
    id: BlockId,
    /// Sprite name in spritesheet.ron, could be moved to IDs later if needed
    #[serde(default)]
    sprite: String,
    /// Drops or defaults to requires
    #[serde(default)]
    drops: Vec<(f32, String)>,
    /// Could be separate type if needed
    #[serde(default)]
    flags: BlockInfoFlags,
    /// For plants, etc... what we grow into
    #[serde(default)]
    growth: Option<GrowthSave>,
    #[serde(default)]
    place_event: Option<String>,
    #[serde(default)]
    mine_event: Option<String>,
    /// What is required to build this item, or nothing if not buildable
    #[serde(default)]
    requires: Vec<String>,
}

impl BlockInfoSave {
    fn convert(
        &self,
        name: &String,
        block_ids: &BlockIds,
        items: &Items,
        events: &EventNames,
    ) -> BlockInfo {
        BlockInfo {
            name: name.clone(),
            sprite: if self.sprite.is_empty() {
                name.clone()
            } else {
                self.sprite.clone()
            },
            drops: if self.drops.is_empty() && !self.requires.is_empty() {
                self.requires
                    .iter()
                    .map(|item_name| {
                        (
                            1.0,
                            items.get_id(item_name).unwrap_or_else(|| {
                                log::error!(
                                    "No item '{}' found in requires for block '{}'",
                                    item_name,
                                    name
                                );
                                ITEM_NONE
                            }),
                        )
                    })
                    .collect()
            } else {
                self.drops
                    .iter()
                    .map(|(frequency, item_name)| {
                        (
                            *frequency,
                            items.get_id(item_name).unwrap_or_else(|| {
                                log::error!(
                                    "No item '{}' found dropped by block '{}'",
                                    item_name,
                                    name
                                );
                                ITEM_NONE
                            }),
                        )
                    })
                    .collect()
            },
            flags: self.flags,
            growth: self.growth.as_ref().map(|growth_save| {
                (
                    growth_save.days * TICKS_PER_HOUR * HOURS_PER_DAY as Tick,
                    *block_ids.get(&growth_save.into).unwrap_or_else(|| {
                        log::error!(
                            "No block '{}' found for growth.into on block '{}'",
                            growth_save.into,
                            name
                        );
                        &BLOCK_NONE
                    }),
                )
            }),
            place_event: self.place_event.as_ref().map(|event_name| {
                events
                    .get(event_name)
                    .unwrap_or_else(|| {
                        log::error!(
                            "No event '{}' found for place_event on block '{}'",
                            event_name,
                            name
                        );
                        &EVENT_NONE
                    })
                    .clone()
            }),
            mine_event: self.mine_event.as_ref().map(|event_name| {
                events
                    .get(event_name)
                    .unwrap_or_else(|| {
                        log::error!(
                            "No event '{}' found for mine_event on block '{}'",
                            event_name,
                            name
                        );
                        &EVENT_NONE
                    })
                    .clone()
            }),
            requires: self
                .requires
                .iter()
                .map(|item_name| {
                    items.get_id(item_name).unwrap_or_else(|| {
                        log::error!("No item '{}' found required by block '{}'", item_name, name);
                        ITEM_NONE
                    })
                })
                .collect(),
        }
    }
}

impl Blocks {
    pub fn add_block(&mut self, block_id: BlockId, block: BlockInfo) {
        self.ids.insert(block.name.clone(), block_id);
        if let Some(_old) = self.infos.insert(block_id, block) {
            log::warn!("Block {} already exists!", block_id);
        }
    }

    pub fn get_info(&self, block_id: &BlockId) -> Option<&BlockInfo> {
        self.infos.get(block_id)
    }

    pub fn get_id(&self, name: &str) -> Option<BlockId> {
        self.ids.get(name).copied()
    }

    pub fn get_content(&self, block_id: &BlockId) -> Option<ContentBlock> {
        Some((*block_id, self.infos.get(block_id)?.flags))
    }

    pub async fn load(&mut self, items: &Items, events: &EventNames) {
        let ron_str = load_file("assets/data/blocks.ron")
            .await
            .expect("Failed to load blocks.ron");

        self.load_from_bytes(&ron_str, items, events);
    }

    fn load_from_bytes(&mut self, bytes: &[u8], items: &Items, events: &EventNames) {
        let blocks_save: BlocksSave =
            ron::de::from_bytes(bytes).expect("Failed to deserialize blocks!");

        for (block_name, block_info_save) in blocks_save.blocks.iter() {
            let id = block_info_save.id;
            if let Some(old_id) = self.ids.insert(block_name.clone(), id) {
                if old_id != id {
                    log::error!(
                        "Block Id changed for block '{}'! Was: {} Now: {}",
                        block_name,
                        old_id,
                        id
                    );
                }
            }
        }

        for (block_name, block_info_save) in blocks_save.blocks.iter() {
            let id = block_info_save.id;
            let block_info = block_info_save.convert(block_name, &self.ids, items, events);

            if !self.infos.contains_key(&id) {
                self.infos.insert(id, block_info);
            } else {
                if self.infos[&id] != block_info {
                    log::warn!("Overwritting block info for block {}", block_name);
                }
            }
        }

        // verify no missing
        for (name, id) in self.ids.iter() {
            if !self.infos.contains_key(id) {
                log::error!("No block info for block '{}' (id: {})", name, id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_blocks() {
        let mut blocks = Blocks::default();
        let items = Items::default();
        let events = EventNames::default();
        let bytes = std::fs::read("assets/data/blocks.ron").expect("Failed to load blocks.ron");

        // NOTE: This will spit out all sorts of errors but still run
        blocks.load_from_bytes(&bytes, &items, &events);

        assert!(blocks.ids.get("stone").is_some_and(|id| *id > 0));
        let stone_id = blocks.ids["stone"];

        let stone_info = &blocks.infos[&stone_id];
        assert_eq!(stone_info.sprite, "stone_floor");
        assert_eq!(stone_info.requires, vec![0]);
        assert!(stone_info.solid());
    }

    #[test]
    fn load_ids() {
        let items = Items::default();
        let events = EventNames::default();

        let original = br#"(blocks: { "dirt": (id: 1), "rock": (solid: true, id: 2) })"#;
        let mut blocks = Blocks::default();
        blocks.load_from_bytes(original, &items, &events);

        // Capture ids as if they'd been baked into a save file.
        let dirt = blocks.get_id("dirt").expect("dirt id");
        assert_eq!(dirt, 1);
        let rock = blocks.get_id("rock").expect("rock id");
        assert_eq!(rock, 2);
    }
}
