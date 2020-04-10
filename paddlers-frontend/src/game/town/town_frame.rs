use crate::gui::ui_state::ClockTick;
use crate::prelude::*;
use quicksilver::prelude::Window;
use specs::WorldExt;
use crate::game::{
    Game,
    town::Town,
};
use crate::view::Frame;
use std::marker::PhantomData;
use quicksilver::graphics::Mesh;

pub (crate) struct TownFrame<'a,'b> {
    phantom: PhantomData<(&'a(), &'b())>,
    // Graphics optimization
    pub background_cache: Option<Mesh>,
}

impl<'a,'b> TownFrame<'a,'b> {
    pub fn new() -> Self {
        TownFrame {
            phantom: PhantomData,
            background_cache: None,
        }
    }
}

impl<'a,'b> Frame for TownFrame<'a,'b> {
    type Error = PadlError;
    type State = Game<'a,'b>;
    type Graphics = Window;
    type Event = PadlEvent;
    fn draw(&mut self, state: &mut Self::State, window: &mut Self::Graphics) -> Result<(),Self::Error> {
        {
            let ul = state.world.fetch::<ScreenResolution>().unit_length();
            let tick = state.world.read_resource::<ClockTick>().0;
            let (asset, town) = 
            (
                &mut state.sprites,
                &state.world.fetch::<Town>(),
            );
            if self.background_cache.is_none() {
                self.background_cache = Some(Mesh::new()); 
                town.render_background(self.background_cache.as_mut().unwrap(), asset.as_mut().expect("assets"), ul);
            }
            window.mesh().extend(self.background_cache.as_ref().unwrap());
            town.render(window, asset.as_mut().expect("assets"), tick, ul)?;
        }
        state.render_town_entities(window)?;
        Ok(())
    }
}