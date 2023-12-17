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

use crate::{GameState, MarkerGameStatePlaying};

pub use self::hex::HexCell;
use self::hex::{on_click, HexClicked, HexMaterial, NonConstructible, SpawnHex};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<HexMaterial>::default())
            .add_event::<HexClicked>()
            .add_systems(
                OnTransition {
                    from: GameState::Menu,
                    to: GameState::Playing,
                },
                setup,
            )
            .add_systems(OnExit(GameState::Playing), unsetup)
            .add_systems(OnEnter(GameState::Playing), update_distances)
            .add_systems(
                Update,
                (((
                    on_click,
                    apply_deferred.in_set(GridFlush), // make sure we flush the grid before updating distances
                    update_distances,
                    update_unconstructible_hexes,
                    apply_deferred.in_set(GridUpdate), // make sure we flush the grid before drawing
                    debug_display_non_constructible_hexes,
                )
                    .chain())
                .run_if(on_event::<HexClicked>()))
                .run_if(in_state(GameState::Playing)),
            );
    }
}

pub const HEX_SIZE: Vec2 = Vec2::new(60., 60.);
pub const MAP_RADIUS: u32 = 10;

#[derive(Debug, Resource)]
pub struct HexGrid {
    pub entities: HashMap<Hex, Entity>,
    pub layout: HexLayout,
    pub bounds: HexBounds,
}

impl HexGrid {
    pub fn hex(&self, hex: &Hex) -> Option<Entity> {
        self.entities.get(hex).copied()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct GridFlush;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct GridUpdate;

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

fn unsetup(mut commands: Commands) {
    commands.remove_resource::<HexGrid>();
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
    grid: Res<HexGrid>,
    mut hexes: Query<(&mut HexCell, &Handle<HexMaterial>)>,
    mut materials: ResMut<Assets<HexMaterial>>,
    mut colored: Local<bool>,
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
                if let Ok((mut cell, hex_material)) = hexes.get_mut(*entity) {
                    // if the hex has content, consider it as a wall and don't add any neighbors
                    if cell.content.is_some() {
                        cell.dist = u32::MAX;
                        continue;
                    }
                    cell.dist = dist;

                    // adds next circle of neighbors to the queue
                    for neighbor in hex.all_neighbors() {
                        // filter out out-of-bounds and inner-bounds hexes
                        if !grid.entities.contains_key(&neighbor) || processed.contains(&neighbor) {
                            continue;
                        }
                        processed.insert(neighbor);
                        next_queue.push(neighbor);
                    }
                    // TODO: debug purposes only, find a better way to color the field
                    if !*colored {
                        let v = dist as f32 / MAP_RADIUS as f32;
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
    *colored = true;
}

fn update_unconstructible_hexes(
    mut commands: Commands,
    grid: Res<HexGrid>,
    hexes: Query<&HexCell>,
) {
    tarjan(
        Hex::ZERO,
        None,
        1,
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut commands,
        &grid,
        &hexes,
    );

    pub fn tarjan(
        hex: Hex,
        parent: Option<Hex>,
        depth: usize,
        lowest_link: &mut HashMap<Hex, usize>,
        current_link: &mut HashMap<Hex, usize>,
        commands: &mut Commands,
        grid: &Res<HexGrid>,
        hexes: &Query<&HexCell>,
    ) {
        current_link.insert(hex, depth);
        lowest_link.insert(hex, depth);

        let all_neighbors = hex.all_neighbors();
        let neighbors: Vec<&Hex> = all_neighbors
            .iter()
            .filter_map(|h| grid.hex(h).map(|e| (h, e)))
            .filter_map(|(h, e)| hexes.get(e).ok().map(|d| (h, d)))
            .filter(|(_h, cell)| cell.content.is_none())
            .map(|(h, _)| h)
            .collect::<Vec<_>>();

        let mut children: usize = 0;

        for neighbor in neighbors {
            // if we discover a new node
            if !current_link.contains_key(neighbor) {
                children += 1;

                tarjan(
                    *neighbor,
                    Some(hex),
                    depth + 1,
                    lowest_link,
                    current_link,
                    commands,
                    grid,
                    hexes,
                );
                let lowest_neighbour_link = lowest_link.get(neighbor).copied().unwrap();
                let lowest_hex_link = lowest_link.get(&hex).copied().unwrap();
                let current_hex_link = current_link.get(&hex).copied().unwrap();

                if lowest_hex_link > lowest_neighbour_link {
                    lowest_link.insert(hex, lowest_neighbour_link);
                }
                if lowest_neighbour_link >= current_hex_link && parent.is_some() {
                    commands
                        .entity(grid.hex(&hex).unwrap())
                        .insert(NonConstructible);
                }
            } else if Some(*neighbor) != parent {
                let lowest_neighbour_link = lowest_link.get(neighbor).copied().unwrap();
                let lowest_hex_link = lowest_link.get(&hex).copied().unwrap();
                if lowest_hex_link > lowest_neighbour_link {
                    lowest_link.insert(hex, lowest_neighbour_link);
                }
            }
        }
        // special case for the root node
        if parent.is_none() && children > 1 {
            commands
                .entity(grid.hex(&hex).unwrap())
                .insert(NonConstructible);
        }
    }
}

pub fn debug_display_non_constructible_hexes(
    grid: Res<HexGrid>,
    hexes: Query<(&Handle<HexMaterial>, Option<&NonConstructible>)>,
    mut materials: ResMut<Assets<HexMaterial>>,
) {
    for hex in grid.bounds.all_coords() {
        if let Some(entity) = grid.entities.get(&hex) {
            if hexes
                .get(*entity)
                .is_ok_and(|(_, maybe_nonconstructible)| maybe_nonconstructible.is_some())
            {
                if let Ok((material, _)) = hexes.get(*entity) {
                    let material = materials.get_mut(material).unwrap();
                    material.color.x = 1.0;
                    material.color.y = 0.0;
                    material.color.z = 0.0;
                }
            }
        }
    }
}
