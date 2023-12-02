mod hex;

use bevy::{
    app::{App, Plugin},
    asset::Assets,
    ecs::system::{Commands, ResMut},
    prelude::*,
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
    sprite::Material2dPlugin,
    utils::{HashMap, HashSet},
};

use hexx::{Hex, HexBounds, HexLayout, PlaneMeshBuilder, Vec2};

use crate::GameState;

pub use self::hex::HexCell;
use self::hex::{spawn_on_click, HexMaterial, SpawnHex, SpawnOnClick};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<HexMaterial>::default())
            .add_event::<SpawnOnClick>()
            .add_systems(
                OnTransition {
                    from: GameState::Menu,
                    to: GameState::Playing,
                },
                setup,
            )
            .add_systems(OnEnter(GameState::Playing), update_distances)
            .add_systems(
                Update,
                (((
                    spawn_on_click,
                    apply_deferred.in_set(GridFlush), // make sure we flush the grid before updating distances
                    update_distances,
                )
                    .chain())
                .run_if(on_event::<SpawnOnClick>()))
                .run_if(in_state(GameState::Playing)),
            );
    }
}

pub const HEX_SIZE: Vec2 = Vec2::new(40., 40.);
pub const MAP_RADIUS: u32 = 15;

#[derive(Debug, Resource)]
pub struct HexGrid {
    pub entities: HashMap<Hex, Entity>,
    pub layout: HexLayout,
    pub bounds: HexBounds,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct GridFlush;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let layout = HexLayout {
        hex_size: HEX_SIZE,
        ..Default::default()
    };
    let mesh = meshes.add(hexagonal_plane(&layout));

    let bounds = HexBounds::new(Hex::ZERO, MAP_RADIUS);
    let entities = bounds
        .all_coords()
        .map(|hex| {
            let position = layout.hex_to_world_pos(hex);
            let entity = commands
                .spawn_empty()
                .add(SpawnHex {
                    position,
                    hex,
                    mesh: mesh.clone(),
                })
                .id();
            (hex, entity)
        })
        .collect();
    let grid = HexGrid {
        entities,
        layout,
        bounds,
    };
    commands.insert_resource(grid);
}

/// Compute a bevy mesh from the layout
fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout).facing(Vec3::Z).build();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs);
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}

// recalculating the distance to the center for all hexes
// to be called when the grid is changed (e.g. when a tower is placed)
fn update_distances(
    mut commands: Commands,
    grid: Res<HexGrid>,
    children: Query<&Children>,
    #[cfg(debug_assertions)] mut hexes: Query<&Handle<HexMaterial>>,
    #[cfg(debug_assertions)] mut materials: ResMut<Assets<HexMaterial>>,
) {
    let center = Hex::ZERO;
    let mut queue = vec![center];
    let mut processed: HashSet<Hex> = HashSet::new();
    let mut dist = 0;

    processed.insert(center);
    while !queue.is_empty() {
        let mut next_queue = Vec::new();
        for hex in queue {
            if let Some(entity) = grid.entities.get(&hex) {
                // if the hex has a child (a tower), assign dist as u32::MAX
                let has_tower = children.get(*entity).map_or(false, |c| !c.is_empty());

                if has_tower {
                    commands.entity(*entity).insert(HexCell { dist: u32::MAX });
                    continue;
                }
                commands.entity(*entity).insert(HexCell { dist });

                // adds next circle of neighbors to the queue
                for neighbor in hex.all_neighbors() {
                    // filter out out-of-bounds and inner-bounds hexes
                    if !grid.entities.contains_key(&neighbor) || processed.contains(&neighbor) {
                        continue;
                    }
                    processed.insert(neighbor);
                    next_queue.push(neighbor);
                }
                #[cfg(debug_assertions)]
                {
                    if let Ok(hex_material) = hexes.get_mut(*entity) {
                        let v = 1.0 - (dist as f32 / MAP_RADIUS as f32);
                        let material = materials.get_mut(hex_material).unwrap();
                        material.color.x = v;
                        material.color.y = v;
                        material.color.z = v;
                    }
                }
            }
        }
        queue = next_queue;
        dist += 1;
    }
}
