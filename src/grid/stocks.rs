use rustc_hash::FxHashMap;

use crate::{entity::Entities, grid::Grid, item::ItemId, tile::Content};

#[derive(Default)]
pub struct Stocks {
    stocks: FxHashMap<ItemId, Stock>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stock {
    available: usize,
    reserved: usize,
    // could add inventories/equiped if desired
}

impl Stock {
    pub fn total(&self) -> usize {
        self.available + self.reserved
    }
}

const DEFAULT_STOCK: Stock = Stock {
    available: 0,
    reserved: 0,
};

impl Stocks {
    pub fn total(&self, item: ItemId) -> usize {
        self.stocks.get(&item).copied().unwrap_or_default().total()
    }
    pub fn available(&self, item: ItemId) -> usize {
        self.stocks
            .get(&item)
            .copied()
            .unwrap_or_default()
            .available
    }
    pub fn add(&mut self, item: ItemId) {
        self.stocks
            .entry(item)
            .or_insert(Stock::default())
            .available += 1;
    }

    pub fn get(&self, item: ItemId) -> &Stock {
        self.stocks.get(&item).unwrap_or(&DEFAULT_STOCK)
    }

    pub fn reserve(&mut self, item: ItemId) {
        let stock = self.stocks.entry(item).or_insert(Stock::default());
        stock.reserved += 1;
        if stock.available == 0 {
            log::error!("Tried to reserve item {} which is not available!", item);
        }
        stock.available = stock.available.saturating_sub(1);
    }

    pub(crate) fn unreserve(&mut self, item: ItemId) {
        if let Some(stock) = self.stocks.get_mut(&item) {
            if stock.reserved == 0 {
                log::error!("Map stock mismatch for item {}", item);
            }
            stock.reserved = stock.reserved.saturating_sub(1);
            stock.available += 1;
        } else {
            log::error!("Tried to remove from non-existant stock for item: {}", item);
        }
    }

    pub fn remove(&mut self, item: ItemId) {
        if let Some(stock) = self.stocks.get_mut(&item) {
            if stock.available == 0 {
                log::error!("Map stock mismatch for item {}", item);
            }
            stock.available = stock.available.saturating_sub(1);
        } else {
            log::error!("Tried to remove from non-existant stock for item: {}", item);
        }
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, u32, Stock> {
        self.stocks.iter()
    }
}

pub fn stocks_verify(grid: &mut Grid, _entities: &Entities) {
    let stocks = &grid.stocks;
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
    // for entity in entities.values() {
    //     for item in entity.base().items.iter() {
    //         new_stocks.add(item.0);
    //     }
    // }

    if stocks.stocks.len() > 0 {
        // check against old
        for item in new_stocks.stocks.keys() {
            let old = stocks.get(*item);
            let new = new_stocks.get(*item);
            if old != new {
                log::error!(
                    "Stock mismatch for {}: stock: {:?} actual: {:?}",
                    item,
                    old,
                    new,
                );
            }
        }
        // check for any not in new
        for item in stocks.stocks.keys() {
            let old = stocks.get(*item);
            let new = new_stocks.get(*item);
            if old != new {
                log::error!(
                    "Stock mismatch for '{}': stock: {:?} actual: {:?}",
                    item,
                    old,
                    new,
                );
            }
        }
    } else {
        // new stocks, set it up
        grid.stocks = new_stocks;
    }
}
