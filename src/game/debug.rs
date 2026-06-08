use crate::{game::Game, gameplay::GameEvent, grid::Pos};

#[derive(Default)]
pub struct DebugVars {
    pub draw_hidden: bool,
}

impl Game {
    /// Handle a single debug command line from the console. Returns a
    /// `GameEvent` if the command produces one. Add new commands here.
    pub fn run_debug_command(&mut self, line: &str) -> Option<GameEvent> {
        let mut parts = line.split_whitespace();
        let cmd = parts.next()?;
        match cmd {
            "reload" => return Some(GameEvent::Reload),
            "goblin" => {
                let x = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                let y = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                self.entities.spawn_goblin(Pos::new(x, y), &mut self.grid);
            }
            "jobs" => {
                dbg!(&self.job_manager);
            }
            "draw_hidden" => {
                self.game_ctx.debug.draw_hidden = true;
            }
            // "events" => {
            //     dbg!(&self.game.game_ctx.events.);
            // }
            "help" => {
                println!("commands: reload, goblin [x] [y], jobs, help");
            }
            "trigger_all" => {
                self.game_ctx.events.trigger_all_timers();
            }
            other => {
                println!("unknown command: {other:?} (try \"help\")");
            }
        }
        None
    }
}
