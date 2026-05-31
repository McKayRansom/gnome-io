
use macroquad::file::load_file;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::BLOCK_NONE,
    event::{EVENT_NONE, EventNames},
    game::{
        Tick,
        time::{HOURS_PER_DAY, TICKS_PER_HOUR},
    },
    item::{ITEM_NONE, Items},
};

use super::{BlockId, BlockType};

type BlockList = FxHashMap<BlockId, BlockType>;
type BlockIds = FxHashMap<String, BlockId>;

#[derive(Default, Serialize, Deserialize)]
pub struct Blocks {
    // theoretically, this won't be changed after startup and doesn't need to be saved...
    #[serde(skip_deserializing, skip_serializing)]
    block_list: BlockList,
    // may have duplicate BlockId entries for aliases
    block_ids: BlockIds,
}

// Deserialize-only struct
#[derive(Debug, Clone, Deserialize)]
struct BlocksSave {
    blocks: FxHashMap<String, BlockTypeSave>,
    aliases: FxHashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GrowthSave {
    // To keep tick math out of blocks.ron this is in days for now...
    days: Tick,
    // String to avoid order mattering in blocks.ron, could also make a 2nd pass but that's annoying...
    into: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BlockTypeSave {
    /// Sprite name in spritesheet.ron, could be moved to IDs later if needed
    #[serde(default)]
    sprite: String,
    /// Drops or defaults to requires
    #[serde(default)]
    drops: Vec<(f32, String)>,
    /// Is pathable, needs refactor...
    #[serde(default)]
    solid: bool,
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

impl BlockTypeSave {
    fn convert(
        &self,
        name: &String,
        block_ids: &BlockIds,
        items: &Items,
        events: &EventNames,
    ) -> BlockType {
        BlockType {
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
                            items.get_item_id(item_name).unwrap_or_else(|| {
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
                            items.get_item_id(item_name).unwrap_or_else(|| {
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
            walkable: !self.solid,
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
                    items.get_item_id(item_name).unwrap_or_else(|| {
                        log::error!("No item '{}' found required by block '{}'", item_name, name);
                        ITEM_NONE
                    })
                })
                .collect(),
        }
    }
}

impl Blocks {
    pub fn add_block(&mut self, block_id: BlockId, block: BlockType) {
        if let Some(_old) = self.block_list.insert(block_id, block) {
            log::warn!("Block {} already exists!", block_id);
        }
    }

    pub fn get_block(&self, block_id: &BlockId) -> Option<&BlockType> {
        self.block_list.get(block_id)
    }

    pub fn get_id(&self, name: &str) -> Option<BlockId> {
        self.block_ids.get(name).copied()
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

        // do block_id's first so we can reference them in terms of growth
        let mut block_id: BlockId = self.block_list.len() as u32 + 1;
        for block_name in blocks_save.blocks.keys() {
            if !self.block_ids.contains_key(block_name) {
                self.block_ids.insert(block_name.clone(), block_id);
                block_id += 1;
            }
        }
        for (block_name, block_type_save) in blocks_save.blocks.iter() {
            let id = self.block_ids[block_name];
            let block_type = block_type_save.convert(block_name, &self.block_ids, items, events);

            if !self.block_list.contains_key(&id) {
                self.block_list.insert(
                    id,
                    block_type,
                );
            } else {
                if self.block_list[&id] != block_type {
                    log::warn!("Overwritting block info for block {}", block_name);
                }
            }
        }

        for (alias, name) in blocks_save.aliases.iter() {
            if !self.block_ids.contains_key(alias) {
            self.block_ids.insert(
                alias.clone(),
                *self.block_ids.get(name).unwrap_or_else(|| {
                    log::error!("No block name '{}' for alias '{}'", name, alias);
                    &BLOCK_NONE
                }),
            );
        }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_items() {
        let mut blocks = Blocks::default();
        let items = Items::default();
        let events = EventNames::default();
        let bytes = std::fs::read("assets/data/blocks.ron").expect("Failed to load blocks.ron");

        // NOTE: This will spit out all sorts of errors but still run
        blocks.load_from_bytes(&bytes, &items, &events);

        assert!(
            blocks
                .block_ids
                .get("stone_block")
                .is_some_and(|id| *id > 0)
        );
        let stone_id = blocks.block_ids["stone_block"];

        let stone_info = &blocks.block_list[&stone_id];
        assert_eq!(stone_info.sprite, "stone");
        assert_eq!(stone_info.requires, vec![0]);
        assert!(!stone_info.walkable);
    }
}
