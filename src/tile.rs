use hecs::Entity;


#[derive(Debug, Clone)]
pub struct Tile {
    pub item: Option<Entity>,
    pub gnome: Option<Entity>,
    pub is_passable: bool,
}

impl Tile {
    pub fn new() -> Tile {
        Tile {
            item: None,
            gnome: None,
            is_passable: true,
        }
    }

    pub fn set_item(&mut self, item: Entity) {
        self.item = Some(item);
    }

    pub fn set_gnome(&mut self, gnome: Entity) {
        self.gnome = Some(gnome);
    }

    pub fn set_passable(&mut self, passable: bool) {
        self.is_passable = passable;
    }
    
    pub(crate) fn is_passable(&self) -> bool {
        self.is_passable
    }
}
