use rustc_hash::FxHashMap;

use crate::{
    entity::Entities, grid::{Grid}, item::ItemId, tile::Content
};

#[derive(Default)]
pub struct Stocks {
    stocks: FxHashMap<ItemId, usize>,
}

impl Stocks {
    pub fn get(&self, item: ItemId) -> usize {
        *self.stocks.get(&item).unwrap_or(&0)
    }

    pub fn add(&mut self, item: ItemId) {
        *self.stocks.entry(item).or_insert(0) += 1;
    }

    pub fn remove(&mut self, item: ItemId) {
        if let Some(stock) = self.stocks.get_mut(&item) {
            if *stock == 0 {
                log::error!("Map stock mismatch for item {}", item);
            }
            *stock = *&stock.saturating_sub(1);
        } else {
            log::error!("Tried to remove from non-existant stock for item: {}", item);
        }
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, u32, usize> {
        self.stocks.iter()
    }
}

pub fn stocks_verify(stocks: &Stocks, grid: &Grid, entities: &Entities) {
    let mut new_stocks: Stocks = Stocks::default();
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            for content in grid.get_tile((x, y).into()).unwrap().iter_content() {
                if let Content::Item(item) = content {
                    new_stocks.add(item.0);
                }
            }
        }
    }
    for entity in entities.values() {
        for item in entity.base().items.iter() {
            new_stocks.add(item.0);
        }
    }

    // check against old
    for (item, new_value) in new_stocks.stocks.iter() {
        if stocks
            .stocks
            .get(item)
            .is_none_or(|old_value| old_value != new_value)
        {
            log::error!(
                "Stock mismatch: stock: {} actual: {}",
                stocks.stocks.get(item).unwrap_or(&0),
                new_value
            );
        }
    }
    // check for any not in new
    for key in stocks.stocks.keys() {
        if !new_stocks.stocks.contains_key(key) && stocks.stocks[key] > 0 {
            log::error!(
                "Stock mismatch for '{}': stock: {} actual: 0",
                key,
                stocks.stocks.get(key).unwrap()
            );
        }
    }
}
