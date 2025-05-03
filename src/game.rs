use hecs::World;

use crate::{
    gnome::Gnome,
    grid::{Grid, Pos},
    job::Job,
};

pub struct Game {
    pub world: World,
    pub grid: Grid,
    pub jobs: Vec<Job>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            world: World::new(),
            grid: Grid::new(Pos::new(10, 10)), // Example grid size
            jobs: Vec::new(),
        }
    }

    pub fn generate() -> Game {
        let mut world = World::new();
        let _g = world.spawn((Gnome::new((5, 5).into()),));
        Game {
            world,
            grid: Grid::new(Pos::new(10, 10)), // Example grid size
            jobs: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // Update game state
        for (_entity, (gnome,)) in self.world.query_mut::<(&mut Gnome,)>() {
            gnome.update(&mut self.grid, &mut self.jobs);
        }
    }

    pub fn spawn_job(&mut self, job: Job) {
        self.jobs.push(job);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_game_creation() {
    //     let game = Game::new();
    //     assert_eq!(game, Game {});
    // }

    #[test]
    fn test_game_update() {
        let mut game = Game::new();
        game.update();
        // Add assertions to check the state after update
    }
}
