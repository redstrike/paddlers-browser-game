//! The game event manager handles events that are generated by SPECS systems.
//!
//! Sometimes, a SPEC system triggers some rare effect that it cannot handle
//! with the state that it usually requires. Always reading that other state
//! as well would cause too broad locking and often blows up parameters that
//! are being passed around.
//! As a solution, the system can send a message with the corresponding event
//! to the game event manager who always operates on the entire game state.
//!
//! Try to keep computations in here short and simple.

use crate::game::{
    components::*, player_info::PlayerInfo, story::StoryAction, units::attackers::Visitor,
    units::attackers::*,
};
use crate::gui::input::UiView;
use crate::gui::ui_state::Now;
use crate::init::quicksilver_integration::{GameState, Signal};
use crate::net::game_master_api::RestApiState;
use crate::prelude::*;
use paddlers_shared_lib::api::story::StoryStateTransition;
use paddlers_shared_lib::prelude::*;
use specs::prelude::*;
use std::sync::mpsc::Sender;

/// The SPECS systems' endpoint
pub type EventPool = Sender<GameEvent>;

/// Coordinates of a village in world map
pub type VillageCoordinate = (i32, i32);

#[derive(Debug, PartialEq, Clone)]
/// This used to be just for `Game<'_,'_>` but now was moved up to `GameState`.
/// It is questionable if that makes sense. Ideally, this would be just `TownEvent`, staying in the scope of a single frame.
/// If anything should go between frames, than it should use the signal + notification publish-subscriber system that is to be created
/// (But how, frames in the same view need to communicate a lot)
pub enum GameEvent {
    HoboSatisfied(Entity),
    HttpBuyProphet,
    LoadVillage(VillageKey),
    SendProphetAttack(VillageCoordinate),
    StoryActions(Vec<StoryAction>),
    SwitchToView(UiView),
}

impl GameState {
    pub fn handle_game_events(&mut self) {
        while let Ok(msg) = self.game.game_event_receiver.try_recv() {
            let result = self.try_handle_event(msg);
            self.game.check(result);
        }
    }
    fn try_handle_event(&mut self, evt: GameEvent) -> PadlResult<()> {
        match evt {
            GameEvent::HoboSatisfied(id) => {
                let now = *self.game.world.fetch::<Now>();
                let resolution = *self.game.world.fetch::<ScreenResolution>();
                let town_world = self.game.town_world_mut();
                let mut rend_store = town_world.write_storage::<Renderable>();
                if let Some(mut rend) = rend_store.get_mut(id) {
                    change_duck_sprite_to_happy(&mut rend);
                }
                std::mem::drop(rend_store);
                let hobo_store = town_world.read_storage::<Visitor>();
                if let Some(hobo) = hobo_store.get(id) {
                    if !hobo.hurried {
                        let mut v_store = town_world.write_storage::<Moving>();
                        if v_store.get(id).is_none() {
                            // hobo currently stopped (in frontend)
                            // => Set it moving again, assuming it has been released by the game-master
                            let moving = release_and_move_visitor(hobo, resolution, now);
                            v_store.insert(id, moving)?;
                        }
                        // Tell backend that release might be required
                        let net_store = town_world.read_storage::<NetObj>();
                        let net_id = net_store.get(id).ok_or(PadlError::dev_err(
                            PadlErrorCode::MissingComponent("NetObj"),
                        ))?;
                        RestApiState::get().http_notify_visitor_satisfied(HoboKey(net_id.id))?;
                    }
                }
            }
            GameEvent::HttpBuyProphet => {
                let player: PlayerInfo = *self.game.player().clone();
                crate::game::town::purchase_prophet(&player)?;
            }
            GameEvent::SendProphetAttack((x, y)) => {
                self.game.send_prophet_attack((x, y))?;
                // TODO: Only confirm if HTTP OK is returned
                // (Probably do this after cleaning pu network and promise handling)
                self.game
                    .confirm_to_user(format!("Attacking village <{}:{}>", x, y))?;
            }
            GameEvent::SwitchToView(view) => {
                self.game.switch_view(view);
            }
            GameEvent::StoryActions(actions) => {
                for a in actions {
                    self.try_handle_story_action(a)?;
                }
            }
            GameEvent::LoadVillage(coordinate) => {
                println!("Go to village {:?}", coordinate);
                // TODO
            }
        }
        Ok(())
    }
    fn try_handle_story_action(&mut self, action: StoryAction) -> PadlResult<()> {
        match action {
            StoryAction::OpenScene(scene, slide) => {
                self.viewer.global_event(
                    &mut self.game,
                    &PadlEvent::Signal(Signal::Scene(scene, slide)),
                )?;
                self.game.switch_view(UiView::Dialogue);
            }
            StoryAction::StoryProgress(new_state) => {
                let t = StoryStateTransition {
                    before: self.game.story_state(),
                    after: new_state,
                };
                RestApiState::get().http_update_story_state(t)?;
                self.viewer
                    .handle_signal(&mut self.game, Signal::NewStoryState(new_state))?;
            }
        }
        Ok(())
    }
}
