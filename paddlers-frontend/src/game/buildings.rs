use specs::prelude::*;
use specs::world::EntitiesRes;
use crate::gui::{
    render::Renderable,
    sprites::{SpriteIndex,WithSprite, DynamicSprite},
    z::Z_BUILDINGS,
    utils::*,
};
use crate::game::{
    Game,
    input::Clickable,
    movement::Position,
    fight::{Range,Aura},
    town::{Town, TileIndex},
    components::*,
};
use paddlers_shared_lib::{
    models::*,
    api::attributes::Attributes,
    graphql_types::*,
};

impl Town {
    pub fn insert_new_bulding(&mut self, entities: &EntitiesRes, lazy: &LazyUpdate, pos: TileIndex, bt: BuildingType) -> Entity {
       self. insert_bulding(entities, lazy, pos, bt, bt.attack_power(), bt.attacks_per_cycle(), bt.range(), crate::wasm_setup::utc_now())
    }

    fn insert_bulding(
        &mut self, 
        entities: &EntitiesRes, 
        lazy: &LazyUpdate, 
        tile_index: TileIndex, 
        bt: BuildingType, 
        ap: Option<i64>, 
        attacks_per_cycle: Option<i64>,  
        range: Option<f32>,
        created: crate::Timestamp,
    ) -> Entity 
    {
        let area = self.tile_area(tile_index);
        let mut builder = 
            lazy.create_entity(entities)
            .with(Position::new(area.pos, area.size, Z_BUILDINGS))
            .with(Renderable {
                kind: match bt {
                    BuildingType::Tree => {
                        let dynsp = DynamicSprite::new_tree(created);
                        RenderVariant::DynImgWithImgBackground(dynsp, SpriteIndex::Grass)
                    },
                    _ => RenderVariant::ImgWithImgBackground(bt.sprite(), SpriteIndex::Grass),
                }
            })
            .with(Clickable);

        if let Some(r) = range {
            builder = builder.with(Range::new(r));
        }

        // No (None) attacks per cycle && Some ap => Aura effect
        if attacks_per_cycle.is_none() && ap.is_some() {
            if let Some(r) = range {
                builder = builder.with(Aura::new(r, ap.unwrap(), tile_index, self))
            }
        }

        match bt {
            BuildingType::BundlingStation => {
                builder = builder.with(
                    EntityContainer::new("Working", bt.capacity())
                );
            },
            _ => { }
        }

        self.place_building(tile_index, bt, builder.entity);

        builder.build()
    }
}

use crate::net::graphql::buildings_query;
impl buildings_query::ResponseData {
    pub (crate) fn create_entities(&self, game: &mut Game) -> Vec<Entity> {
        self.village.buildings
            .iter()
            .map(|u|{u.create_entity(game)})
            .collect()
    }
}

impl buildings_query::BuildingsQueryVillageBuildings {
    fn create_entity(&self, game: &mut Game) -> Entity {
        let coordinates = (self.x as usize,self.y as usize);
        let maybe_range = self.building_range.map(|f| f as f32);
        let maybe_ap = self.attack_power.map(|f| f as i64);
        let bt = match self.building_type {
            buildings_query::BuildingType::RED_FLOWERS => BuildingType::RedFlowers,
            buildings_query::BuildingType::BLUE_FLOWERS => BuildingType::BlueFlowers,
            buildings_query::BuildingType::TREE => BuildingType::Tree,
            buildings_query::BuildingType::BUNDLING_STATION => BuildingType::BundlingStation,
            _ => panic!("Unexpected BuildingType"),
        };
        let created = GqlTimestamp::from_string(&self.creation).unwrap().0;

        let entities = game.world.entities();
        let lazy = game.world.read_resource::<LazyUpdate>();
        let mut town = game.world.write_resource::<Town>();
        town.insert_bulding(&entities, &lazy, coordinates, bt, maybe_ap, self.attacks_per_cycle, maybe_range, created)
    }
}