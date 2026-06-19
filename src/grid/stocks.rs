use crate::{
    entity::Entities, grid::{Grid, Stocks}, item::ItemId, tile::Content
};

pub fn stocks_add(stocks: &mut Stocks, item: ItemId) {
    *stocks.entry(item).or_insert(0) += 1;
}

pub fn stocks_remove(stocks: &mut Stocks, item: ItemId) {
    if let Some(stock) = stocks.get_mut(&item) {
        if *stock == 0 {
            log::error!("Map stock mismatch for item {}", item);
        }
        *stock = *&stock.saturating_sub(1);
    } else {
        log::error!("Tried to remove from non-existant stock for item: {}", item);
    }
}

pub fn stocks_verify(stocks: &Stocks, grid: &Grid, entities: &Entities) {
    let mut new_stocks: Stocks = Stocks::default();
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            for content in grid.get_tile((x, y).into()).unwrap().iter_content() {
                if let Content::Item(item) = content {
                    stocks_add(&mut new_stocks, item.0);
                }
            }
        }
    }
    for entity in entities.values() {
        for item in entity.base().items.iter() {
            stocks_add(&mut new_stocks, item.0);
        }
    }

    // check against old
    for (item, new_value) in new_stocks.iter() {
        if stocks
            .get(item)
            .is_none_or(|old_value| old_value != new_value)
        {
            log::error!(
                "Stock mismatch: stock: {} actual: {}",
                stocks.get(item).unwrap_or(&0),
                new_value
            );
        }
    }
    // check for any not in new
    for key in stocks.keys() {
        if !new_stocks.contains_key(key) && stocks[key] > 0 {
            log::error!(
                "Stock mismatch for '{}': stock: {} actual: 0",
                key,
                stocks.get(key).unwrap()
            );
        }
    }
}
