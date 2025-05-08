use std::collections::HashMap;

use crate::{block::BlockId, tileset::Sprite};

pub type ItemId = u32;

pub struct ItemType {
    pub _sprite: Sprite,
    pub _builds: Option<BlockId>,
    // food value?
}

impl ItemType {
    pub fn builds(sprite: Sprite, block: BlockId) -> Self {
        Self {
            _sprite: sprite,
            _builds: Some(block),
        }
    }
    pub fn new(sprite: Sprite) -> Self {
        Self {
            _sprite: sprite,
            _builds: None,
        }
    }
}

pub struct Items {
    item_list: HashMap<ItemId, ItemType>,
}

impl Items {
    pub fn new() -> Self {
        Items {
            item_list: HashMap::new(),
        }
    }

    pub fn add_item(&mut self, id: ItemId, item: ItemType) {
        if let Some(_old) = self.item_list.insert(id, item) {
            log::warn!("Item {} already exists!", id);
        }
    }

    pub fn get_item(&self, id: &ItemId) -> Option<&ItemType> {
        self.item_list.get(&id)
    }
}
