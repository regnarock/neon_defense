use std::time::Duration;

use crate::inventory::{self};
use crate::inventory::{Inventory, SpawnInventory};
use crate::random::RandomDeterministic;
use crate::window::WindowSize;
use crate::{GameState, MarkerGameStatePlaying};
use bevy::ecs::system::{EntityCommand, SystemParam, SystemState};

use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::utils::HashMap;
use bevy_easings::{Ease, EaseFunction, EasingType};
use rand::seq::SliceRandom;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(inventory::InventoryPlugin::<Building>::default())
            .init_resource::<BuildingInventory>()
            .add_systems(
                OnEnter(GameState::Playing),
                (create_assets, spawn_layout).chain(),
            )
            .add_systems(
                Update,
                update_anchor_position
                    .run_if(resource_changed::<WindowSize>())
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
pub struct BuildingInventory {
    pub(crate) state: SystemState<GetNextBuildingParams<'static, 'static>>,
}

#[derive(SystemParam)]
pub(crate) struct GetNextBuildingParams<'w, 's> {
    command: Commands<'w, 's>,
    q_inventory: Query<
        'w,
        's,
        (
            &'static mut RandomDeterministic,
            &'static mut crate::inventory::Inventory<Building>,
        ),
    >,
    q_buildings: Query<'w, 's, &'static Building>,
}

impl FromWorld for BuildingInventory {
    fn from_world(world: &mut World) -> Self {
        BuildingInventory {
            state: SystemState::new(world),
        }
    }
}

impl BuildingInventory {
    pub fn next(&mut self, world: &mut World) -> Option<Building> {
        let mut params = self.state.get_mut(world);
        let (mut rng, mut inventory) = params.q_inventory.single_mut();

        let Some(first_item) = inventory.items.front().cloned() else {
            return None;
        };
        let Ok(_item_to_build) = params.q_buildings.get(first_item) else {
            return None;
        };
        // TODO: check if we can build item_to_build (cooldown, space available, currency, ...)
        // TODO: send an event if not possible.
        // TODO: pay "price" ?
        inventory.items.pop_front();

        let new_building = get_random_building(&mut rng);
        let new_item = params
            .command
            .spawn((new_building, MarkerGameStatePlaying))
            .id();

        inventory.items.push_back(new_item);

        // TODO: reuse that entity to merge it with turret entity ?
        world.despawn(first_item);

        self.state.apply(world);
        Some(new_building)
    }
}

#[derive(Resource)]
pub struct VisualAssets {
    pub mesh_def: HashMap<BuildingMesh, Mesh2dHandle>,
    pub size_def: HashMap<BuildingSize, f32>,
    pub color_def: HashMap<BuildingColor, Handle<ColorMaterial>>,
}

pub(crate) fn create_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(VisualAssets {
        mesh_def: [
            (
                BuildingMesh::Triangle,
                meshes
                    .add(
                        Mesh::new(PrimitiveTopology::TriangleList)
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_POSITION,
                                vec![[-0.5, -0.5, 0.0], [0.0, 0.5, 0.0], [0.5, -0.5, 0.0]],
                            )
                            .with_indices(Some(Indices::U32(vec![0, 1, 2]))),
                    )
                    .into(),
            ),
            (
                BuildingMesh::Circle,
                meshes.add(Mesh::from(shape::Circle::default())).into(),
            ),
            (
                BuildingMesh::Quad,
                meshes.add(Mesh::from(shape::Quad::default())).into(),
            ),
        ]
        .into(),
        size_def: [
            (BuildingSize::Big, 1f32),
            (BuildingSize::Medium, 0.75f32),
            (BuildingSize::Small, 0.5f32),
        ]
        .into(),
        color_def: [
            (
                BuildingColor::Black,
                materials.add(ColorMaterial::from(Color::BLACK)),
            ),
            (
                BuildingColor::White,
                materials.add(ColorMaterial::from(Color::WHITE)),
            ),
            (
                BuildingColor::Pink,
                materials.add(ColorMaterial::from(Color::PINK)),
            ),
            (
                BuildingColor::Blue,
                materials.add(ColorMaterial::from(Color::BLUE)),
            ),
        ]
        .into(),
    });
}

const ITEM_VISUAL_SIZE: f32 = 64f32;
const PADDING: f32 = 10f32;

pub(crate) fn spawn_layout(mut commands: Commands, window_size: ResMut<WindowSize>) {
    let mut rng = crate::random::RandomDeterministic::new_from_seed(0);
    let inventory = {
        let mut inventory = vec![];
        for _ in 0..6 {
            inventory.push(
                commands
                    .spawn((get_random_building(&mut rng), MarkerGameStatePlaying))
                    .id(),
            );
        }
        inventory
    };
    let anchor_point = Vec3::new(
        -window_size.size.x / 2f32 + ITEM_VISUAL_SIZE / 2f32 + PADDING,
        -window_size.size.y / 2f32 + (ITEM_VISUAL_SIZE + PADDING) * 5.5f32 + PADDING,
        0f32,
    );

    commands
        .spawn_empty()
        .add(SpawnInventory::<Building>::new(
            inventory,
            inventory::InventoryConfiguration {
                positions: positions_from_anchor_point(anchor_point),
            },
        ))
        .insert(SpatialBundle::default())
        .insert(Transform::from_translation(vec3(-100.0, 0.0, 0.0)).ease_to(
            Transform::from_translation(vec3(0.0, 0.0, 0.0)),
            EaseFunction::QuadraticIn,
            EasingType::Once {
                duration: Duration::from_secs_f32(0.5f32),
            },
        ))
        .insert(MarkerGameStatePlaying)
        .insert(RandomDeterministic::new_from_seed(0));
}

fn positions_from_anchor_point(anchor_point: Vec3) -> Vec<Vec3> {
    vec![
        anchor_point - Vec3::new(0f32, (ITEM_VISUAL_SIZE + PADDING) * 5f32, 0f32),
        anchor_point - Vec3::new(0f32, (ITEM_VISUAL_SIZE + PADDING) * 4f32, 0f32),
        anchor_point - Vec3::new(0f32, (ITEM_VISUAL_SIZE + PADDING) * 3f32, 0f32),
        anchor_point - Vec3::new(0f32, (ITEM_VISUAL_SIZE + PADDING) * 2f32, 0f32),
        anchor_point - Vec3::new(0f32, ITEM_VISUAL_SIZE + PADDING, 0f32),
        anchor_point,
    ]
}

pub(crate) fn update_anchor_position(
    window_size: ResMut<WindowSize>,
    mut q_inventory: Query<&mut Inventory<Building>>,
) {
    let anchor_point: Vec3 = Vec3::new(
        -window_size.size.x / 2f32 + ITEM_VISUAL_SIZE / 2f32 + PADDING,
        -window_size.size.y / 2f32 + (ITEM_VISUAL_SIZE + PADDING) * 5.5f32 + PADDING,
        0f32,
    );
    q_inventory.for_each_mut(|mut inventory| {
        inventory.positions = positions_from_anchor_point(anchor_point);
    });
}

#[derive(Reflect, Component, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Building {
    mesh: BuildingMesh,
    size: BuildingSize,
    color: BuildingColor,
}

#[derive(Reflect, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BuildingMesh {
    Triangle,
    Circle,
    Quad,
}
#[derive(Reflect, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BuildingSize {
    Small,
    Medium,
    Big,
}
#[derive(Reflect, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BuildingColor {
    Black,
    White,
    Pink,
    Blue,
}

impl inventory::ItemSpriteBuilder for Building {
    type C = BuildingItemSpriteBuilder;
    fn build_sprite(&self) -> Self::C {
        BuildingItemSpriteBuilder { building: *self }
    }
}

pub struct BuildingItemSpriteBuilder {
    pub building: Building,
}

impl EntityCommand for BuildingItemSpriteBuilder {
    fn apply(self, id: Entity, world: &mut World) {
        let assets = world.get_resource::<VisualAssets>().unwrap();
        let visual = MaterialMesh2dBundle {
            mesh: assets.mesh_def[&self.building.mesh].clone(),
            transform: Transform::default().with_scale(Vec3::splat(
                ITEM_VISUAL_SIZE * assets.size_def[&self.building.size],
            )),
            material: assets.color_def[&self.building.color].clone(),
            ..default()
        };
        let mut q_inventory: SystemState<Query<Entity, With<Inventory<Building>>>> =
            SystemState::new(world);
        let inventory_entity = q_inventory.get_mut(world).single();
        world
            .entity_mut(id)
            .set_parent(inventory_entity)
            .insert(visual)
            .insert(MarkerGameStatePlaying);
    }
}

pub fn get_random_building(rng: &mut crate::random::RandomDeterministic) -> Building {
    let choices_mesh = [
        (BuildingMesh::Triangle, 2),
        (BuildingMesh::Circle, 2),
        (BuildingMesh::Quad, 2),
    ];
    let choices_size = [
        (BuildingSize::Big, 1),
        (BuildingSize::Medium, 2),
        (BuildingSize::Small, 1),
    ];
    let choices_color = [
        (BuildingColor::Black, 5),
        (BuildingColor::White, 5),
        (BuildingColor::Pink, 1),
        (BuildingColor::Blue, 1),
    ];
    let building = Building {
        mesh: choices_mesh
            .choose_weighted(&mut rng.random, |i| i.1)
            .unwrap()
            .0,
        size: choices_size
            .choose_weighted(&mut rng.random, |i| i.1)
            .unwrap()
            .0,
        color: choices_color
            .choose_weighted(&mut rng.random, |i| i.1)
            .unwrap()
            .0,
    };
    building
}
