use crate::{
    entity::{
        BaseEntity, EntityBehaviour, EntityId, Faction,
        gnome::{GNOME_FACTION, GNOME_SPEED},
    },
    game::time::hours,
    grid::{PathOutcome, Pos},
    tile::Content,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Goblin {
    pub base: BaseEntity,
    #[serde(default)]
    pub fighting: bool,
}

pub const GOBLIN_FACTION: Faction = 2;

const BASE_HEALTH: u8 = 10;
const BASE_FOOD: u16 = hours(20);

impl Goblin {
    pub fn new(id: EntityId, pos: Pos, grid: &mut crate::grid::Grid) -> Goblin {
        grid.gnome_enter(pos, (GOBLIN_FACTION, id));

        Goblin {
            base: BaseEntity {
                id,
                faction: GOBLIN_FACTION,
                pos,
                health: BASE_HEALTH,
                food: BASE_FOOD,
                ..Default::default()
            },
            fighting: false,
        }
    }
}

impl EntityBehaviour for Goblin {
    fn update(
        &mut self,
        grid: &mut crate::grid::Grid,
        _ctx: &mut crate::game::GameCtx,
    ) -> Option<super::EntityAction> {
        if let Some(action) = self.base.update(grid) {
            return Some(action);
        }
        if self.base.timer > 0 {
            return None;
        }
        self.fighting = false;
        match grid.find_content(
            self.base.pos,
            Content::Entity((GNOME_FACTION, 0)),
            GOBLIN_FACTION,
        ) {
            PathOutcome::Path(path) => {
                self.base.move_to(path[0], GNOME_SPEED, grid);
                None
            }
            PathOutcome::Reached(attack_pos) => {
                self.base.timer = super::FIGHT_TIME;
                self.fighting = true;
                Some(super::EntityAction::Attack({
                    let Content::Entity((_faction, id)) = grid
                        .get_tile(attack_pos)
                        .unwrap()
                        .find(&Content::Entity((GNOME_FACTION, 0)))
                        .unwrap()
                    else {
                        panic!();
                    };
                    id
                }))
            }
            PathOutcome::NoPath => {
                self.base.move_random(grid);
                None
            }
        }
    }

    fn die(&mut self, grid: &mut crate::grid::Grid, _ctx: &mut crate::game::GameCtx) {
        self.base.die(grid);
    }

    fn attacked(&mut self) {
        self.base.attacked();
    }

    fn base(&self) -> &BaseEntity {
        &self.base
    }
}
