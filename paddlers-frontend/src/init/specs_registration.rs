use crate::game::{
    components::*, fight::Aura, player_info::PlayerInfo, story::entity_trigger::EntityTrigger,
    town::nests::Nest, town::DefaultShop, town::Town, town_resources::TownResources,
    units::attackers::Visitor, units::hobos::Hobo, units::workers::Worker, visits::attacks::Attack,
};
use crate::gui::input::drag::Drag;
use crate::gui::ui_state::*;
use crate::prelude::*;
use crate::view::entry_view;
use paddle::utc_now;
use specs::prelude::*;

pub(super) fn insert_global_resources(
    world: &mut World,
    resolution: ScreenResolution,
    player_info: PlayerInfo,
) {
    world.insert(ClockTick(0));
    world.insert(Now(utc_now()));
    world.insert(UiState::new());
    world.insert(ViewState::new());
    world.insert(player_info);
    world.insert(resolution);
    let view = entry_view(player_info.story_state());
    world.insert(view);
}

pub fn register_global_components(world: &mut World) {
    world.register::<NetObj>();
    register_graphic_components(world);
    register_ui_components(world);

    // Map view
    world.register::<MapPosition>();
    world.register::<VillageMetaInfo>();

    // Visits view
    world.register::<Attack>();
}

pub fn insert_town_resources(world: &mut World, player_info: PlayerInfo, town: Town) {
    world.insert(DefaultShop::new(&player_info));
    world.insert(Drag::default());
    world.insert(Now(utc_now()));
    world.insert(TownResources::default());
    world.insert(UiState::new());
    world.insert(ViewState::new());
    world.insert(player_info);
    world.insert(town);
}

/// All components used in the town view
pub fn register_town_components(world: &mut World) {
    world.register::<Aura>();
    world.register::<Building>();
    world.register::<EntityContainer>();
    world.register::<ForestComponent>();
    world.register::<Health>();
    world.register::<Hobo>();
    world.register::<Level>();
    world.register::<Mana>();
    world.register::<Moving>();
    world.register::<Nest>();
    world.register::<Range>();
    world.register::<StatusEffects>();
    world.register::<TargetPosition>();
    world.register::<Visitor>();
    world.register::<Worker>();

    register_graphic_components(world);
    register_ui_components(world);
    world.register::<NetObj>();
}

fn register_ui_components(world: &mut World) {
    world.register::<Clickable>();
    world.register::<EntityTrigger>();
    world.register::<UiMenu>();
}
fn register_graphic_components(world: &mut World) {
    world.register::<AnimationState>();
    world.register::<Position>();
    world.register::<Renderable>();
}
