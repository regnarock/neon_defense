use crate::inventory::{self, Inventory};
use bevy::ecs::system::EntityCommand;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::utils::HashMap;
use rand::seq::SliceRandom;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(inventory::InventoryPlugin::<ItemType>::default());
        app.add_systems(Startup, (create_assets, spawn_layout).chain());
    }
}

#[derive(Resource)]
pub struct VisualAssets {
    pub mesh_def: HashMap<MeshType, Mesh2dHandle>,
    pub size_def: HashMap<SizeType, f32>,
    pub color_def: HashMap<ColorType, Handle<ColorMaterial>>,
}

pub(crate) fn create_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(VisualAssets {
        mesh_def: [
            (
                MeshType::Triangle,
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
                MeshType::Circle,
                meshes.add(Mesh::from(shape::Circle::default())).into(),
            ),
            (
                MeshType::Quad,
                meshes.add(Mesh::from(shape::Quad::default())).into(),
            ),
        ]
        .into(),
        size_def: [
            (SizeType::Big, 1f32),
            (SizeType::Medium, 0.75f32),
            (SizeType::Small, 0.5f32),
        ]
        .into(),
        color_def: [
            (
                ColorType::Black,
                materials.add(ColorMaterial::from(Color::BLACK)),
            ),
            (
                ColorType::White,
                materials.add(ColorMaterial::from(Color::WHITE)),
            ),
            (
                ColorType::Pink,
                materials.add(ColorMaterial::from(Color::PINK)),
            ),
            (
                ColorType::Blue,
                materials.add(ColorMaterial::from(Color::BLUE)),
            ),
        ]
        .into(),
    });
}

const ITEM_VISUAL_SIZE: f32 = 64f32;

pub(crate) fn spawn_layout(mut commands: Commands) {
    let mut rng = crate::random::RandomDeterministic::new_from_seed(0);
    let inventory = vec![
        commands.spawn(get_random_item(&mut rng)).id(),
        commands.spawn(get_random_item(&mut rng)).id(),
        commands.spawn(get_random_item(&mut rng)).id(),
        commands.spawn(get_random_item(&mut rng)).id(),
        commands.spawn(get_random_item(&mut rng)).id(),
        commands.spawn(get_random_item(&mut rng)).id(),
    ]
    .into();
    commands.spawn((
        Inventory::<ItemType> {
            items: inventory,
            ..default()
        },
        rng,
        inventory::InventoryVisualDef {
            positions: vec![
                vec3(-350f32, 0f32, 0f32),
                vec3(-350f32, ITEM_VISUAL_SIZE + 10f32, 0f32),
                vec3(-350f32, (ITEM_VISUAL_SIZE + 10f32) * 2f32, 0f32),
                vec3(-350f32, (ITEM_VISUAL_SIZE + 10f32) * 3f32, 0f32),
                vec3(-350f32, (ITEM_VISUAL_SIZE + 10f32) * 4f32, 0f32),
                vec3(-350f32, (ITEM_VISUAL_SIZE + 10f32) * 5f32, 0f32),
            ],
        },
    ));
}

#[derive(Component, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ItemType(MeshType, SizeType, ColorType);

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum MeshType {
    Triangle,
    Circle,
    Quad,
}
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum SizeType {
    Small,
    Medium,
    Big,
}
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum ColorType {
    Black,
    White,
    Pink,
    Blue,
}

impl inventory::CommandVisualBuilder for ItemType {
    type C = CreateItemDefVisual;
    fn command_to_create_visual(&self) -> Self::C {
        CreateItemDefVisual { item_type: *self }
    }
}

pub struct CreateItemDefVisual {
    pub item_type: ItemType,
}

impl EntityCommand for CreateItemDefVisual {
    fn apply(self, id: Entity, world: &mut World) {
        let assets = world.get_resource::<VisualAssets>().unwrap();
        let visual = MaterialMesh2dBundle {
            mesh: assets.mesh_def[&self.item_type.0].clone(),
            transform: Transform::default().with_scale(Vec3::splat(
                ITEM_VISUAL_SIZE * assets.size_def[&self.item_type.1],
            )),
            material: assets.color_def[&self.item_type.2].clone(),
            ..default()
        };
        world.entity_mut(id).insert(visual);
    }
}

pub fn get_random_item(rng: &mut crate::random::RandomDeterministic) -> ItemType {
    let choices_mesh = [
        (MeshType::Triangle, 2),
        (MeshType::Circle, 2),
        (MeshType::Quad, 2),
    ];
    let choices_size = [
        (SizeType::Big, 1),
        (SizeType::Medium, 2),
        (SizeType::Small, 1),
    ];
    let choices_color = [
        (ColorType::Black, 5),
        (ColorType::White, 5),
        (ColorType::Pink, 1),
        (ColorType::Blue, 1),
    ];
    let item_type = ItemType(
        choices_mesh
            .choose_weighted(&mut rng.random, |i| i.1)
            .unwrap()
            .0,
        choices_size
            .choose_weighted(&mut rng.random, |i| i.1)
            .unwrap()
            .0,
        choices_color
            .choose_weighted(&mut rng.random, |i| i.1)
            .unwrap()
            .0,
    );
    item_type
}
