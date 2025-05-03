use crate::tileset::Tileset;

pub struct Context {
    pub tileset: Tileset,
}

impl Context {
    pub async fn new() -> Self {
        Self {
            tileset: Tileset::new().await,
        }
    }
}
