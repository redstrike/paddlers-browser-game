//! The game event manager handles events that are generated by SPECS systems.
//! 
//! Sometimes, a SPEC system triggers some rare effect that it cannot handle
//! with the state that it usually requires. Always reading that other state
//! as well would cause too broad locking and often blows up parameters that
//! are being passed around.
//! As a solution, the system can send a message with the corresponding evenet 
//! to the game event manager who always operates on the entire game state.
//! 
//! Try to keep computations in here short and simple.

use std::sync::mpsc::Sender;
use specs::prelude::*;
use crate::prelude::*;
use crate::game::{
    Game,
    components::*,
    units::attackers::change_duck_sprite_to_happy,
    player_info::PlayerInfo,
    };

/// The SPECS systems' endpoint
pub type EventPool = Sender<GameEvent>;

#[derive(Debug, PartialEq, Clone)]
pub enum GameEvent {
    HoboSatisfied(Entity),
    HttpBuyProphet,
    SendProphetAttack((i32,i32)),
}

impl Game<'_,'_> {
    pub fn handle_game_events(&mut self) {
        while let Ok(msg) = self.game_event_receiver.try_recv() {
            let result = self.try_handle_event(msg);
            self.check(result);
        }
    }
    fn try_handle_event(&mut self, evt: GameEvent) -> PadlResult<()> {
        match evt {
            GameEvent::HoboSatisfied(id) => {
                let mut rend_store = self.world.write_storage::<Renderable>();
                if let Some(mut rend) = rend_store.get_mut(id) {
                    change_duck_sprite_to_happy(&mut rend);
                }
            },
            GameEvent::HttpBuyProphet => {
                let player : PlayerInfo = *self.player().clone();
                crate::game::town::purchase_prophet(&mut *self.rest(), &player)?;
            },
            GameEvent::SendProphetAttack((x,y)) => {
                if self.town().idle_prophets.len() == 0 {
                    return PadlErrorCode::NotEnoughUnits.usr();
                }
                self.send_prophet_attack((x,y))?;
                // TODO: Only confirm if HTTP OK is returned 
                // (Probably do this after cleaning pu network and promise handling)
                self.confirm_to_user(format!("Attacking village <{}:{}>", x, y));
            }
        }
        Ok(())
    }
}