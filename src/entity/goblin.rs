use crate::{
    entity::{
        BaseEntity, EntityBehaviour, EntityId, Faction, HIDDEN_FACTION,
        gnome::{GNOME_FACTION, GNOME_SPEED},
    },
    game::time::hours,
    grid::{path::PathOutcome, Pos},
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
const VISIBLE_DISTANCE: usize = 10;

impl Goblin {
    pub fn new(id: EntityId, pos: Pos, _grid: &mut crate::grid::Grid) -> Goblin {
        // don't enter the grid yet, we may want to be hidden...
        // grid.entity_enter(pos, (GOBLIN_FACTION, id));

        Goblin {
            base: BaseEntity {
                id,
                faction: HIDDEN_FACTION,
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
        // reset hiding status
        self.base.faction = GOBLIN_FACTION;

        // TODO: Find path including tunneling
        match grid.find_content(
            self.base.pos,
            Content::Entity((GNOME_FACTION, 0)),
            GOBLIN_FACTION,
        ) {
            PathOutcome::Path(path) => {
                // hide
                if path.len() > VISIBLE_DISTANCE {
                    self.base.faction = HIDDEN_FACTION;
                }

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
                // TODO: Remove this case...
                self.base.faction = HIDDEN_FACTION;
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
