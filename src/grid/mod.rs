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

use bevy_mod_picking::prelude::PointerButton;
use hexx::{Hex, HexBounds, HexLayout, PlaneMeshBuilder, Vec2};

use crate::{entities::portal::SpawnPortalCmd, entities::turret::SpawnTurretCmd, GameState};

pub use self::hex::HexCell;
use self::hex::{HexClicked, HexMaterial, SpawnHexCmd};

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
            .add_systems(OnEnter(GameState::Playing), update_distances)
            .add_systems(
                Update,
                // All the systems to execute while the game is playing
                (
                    // Execute this chain after each click on the grid
                    ((
                        on_hex_clicked,
                        detect_despawned_grid_content,
                        //FIXME: find a better solution
                        clear_unconstructible_hexes, // remove all nonconstructibletags before recalculating
                        apply_deferred.in_set(GridFlush), // make sure we flush the grid before updating distances
                        update_distances,
                        update_unconstructible_hexes,
                        apply_deferred.in_set(GridUpdate), // make sure we flush the grid before drawing
                        debug_display_non_constructible_hexes,
                    )
                        .chain())
                    .run_if(on_event::<HexClicked>())
                )
                .run_if(in_state(GameState::Playing)),
            );
    }
}

pub const HEX_SIZE: Vec2 = Vec2::new(60., 60.);
pub const MAP_RADIUS: u32 = 10;

#[derive(Debug, Resource)]
pub struct HexGrid {
    entities: HashMap<Hex, Entity>,
    pub layout: HexLayout,
    pub bounds: HexBounds,
}

impl HexGrid {
    pub fn hex_to_entity(&self, hex: &Hex) -> Option<&Entity> {
        self.entities.get(hex)
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
                .add(SpawnHexCmd {
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

#[derive(Debug, Default, Component)]
pub struct NonConstructible;

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
    mut hexes: Query<(&mut HexCell, &Handle<HexMaterial>, Option<&Children>)>,
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
                if let Ok((mut cell, hex_material, content)) = hexes.get_mut(*entity) {
                    // if the hex has content, consider it as a wall and don't add any neighbors
                    if content.is_some() {
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
                    // FIXME: debug purposes only, find a better way to color the field
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

fn detect_despawned_grid_content(
    mut command: Commands,
    hexes: Query<(Entity, &Children), With<HexCell>>,
    entities: Query<Entity, Without<HexCell>>,
) {
    hexes.for_each(|(e, children)| {
        if !entities.contains(*children.first().unwrap()) {
            println!(
                "Detected a hex still parenting a building that's now destroyed, cleaned children."
            );
            command.entity(e).clear_children();
        }
    });
}

fn clear_unconstructible_hexes(
    mut command: Commands,
    hexes: Query<Entity, (Without<Children>, With<NonConstructible>, With<HexCell>)>,
) {
    hexes.for_each(|e| {
        command.entity(e).remove::<NonConstructible>();
    });
}

fn update_unconstructible_hexes(
    mut commands: Commands,
    grid: Res<HexGrid>,
    hexes: Query<Entity, Without<Children>>,
) {
    // detect hexes that if constructed upon would prevent from having a path to the center and mark them as NonConstructible
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

    /// Recursive depth-first search function for Tarjan's strongly connected components algorithm.
    /// This function is used to identify non-constructible entities in a grid of hexagonal cells.
    // TODO: move the bevy side out of this function and isolate it
    pub fn tarjan(
        hex: Hex,                               // Current hexagon being processed
        parent: Option<Hex>,                    // Parent hexagon in the current path from root
        depth: usize,                           // Depth of current recursion level
        lowest_link: &mut HashMap<Hex, usize>,  // lowest max depth reached per node
        current_link: &mut HashMap<Hex, usize>, // current depth per node
        commands: &mut Commands,
        grid: &Res<HexGrid>,
        hexes: &Query<Entity, Without<Children>>, // Query for accessing hexagon cells without content
    ) {
        // Mark the current hexagon as processed and update its lowest link number.
        current_link.insert(hex, depth);
        lowest_link.insert(hex, depth);

        // Get all neighboring hexagons that are not yet processed.
        let all_neighbors = hex.all_neighbors();
        let neighbors: Vec<&Hex> = all_neighbors
            .iter()
            .filter(|h|
                // check that the neighbors exist in the grid and they can be queried (=> meaning that they have no content)
                grid.hex_to_entity(h).and_then(|&e| hexes.get(e).ok()).is_some())
            .collect::<Vec<_>>();

        // Initialize the count of children for the current hexagon.
        let mut children: usize = 0;

        // Iterate through all neighboring hexagons and recursively call tarjan function if needed.
        for neighbor in neighbors {
            // If we discover a new node, increment the number of children and recurse.
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

                // Update the lowest link number for the current hexagon if necessary.
                if lowest_hex_link > lowest_neighbour_link {
                    lowest_link.insert(hex, lowest_neighbour_link);
                }
                // If the parent is set and the neighbor's lowest link number is greater than or equal to the current hexagon's link number, mark the current hexagon as non-constructible.
                if lowest_neighbour_link >= current_hex_link && parent.is_some() {
                    if let Some(&e) = grid.hex_to_entity(&hex) {
                        commands.entity(e).insert(NonConstructible);
                    }
                }
            } else if Some(*neighbor) != parent {
                // If the neighbor is already processed, update the lowest link number for the current hexagon if necessary.
                let lowest_neighbour_link = lowest_link.get(neighbor).copied().unwrap();
                let lowest_hex_link = lowest_link.get(&hex).copied().unwrap();
                if lowest_hex_link > lowest_neighbour_link {
                    lowest_link.insert(hex, lowest_neighbour_link);
                }
            }
        }

        // Special case for the root node: If there are more than one child and the current hexagon is not the root, mark it as non-constructible.
        if parent.is_none() && children > 1 {
            if let Some(&e) = grid.hex_to_entity(&hex) {
                commands.entity(e).insert(NonConstructible);
            }
        }
    }
}

pub fn on_hex_clicked(
    mut commands: Commands,
    mut clicks: EventReader<HexClicked>,
    hexes: Query<(&Transform, &Handle<HexMaterial>), Without<NonConstructible>>,
    mut materials: ResMut<Assets<HexMaterial>>,
    grid: Res<HexGrid>,
) {
    for click in clicks.read() {
        if let Ok((transform, material)) = hexes.get(click.target) {
            // early return if we clicked on an unselected hex
            if materials.get_mut(material).unwrap().is_selected == 0. {
                return;
            }
            let spawned_id = match click.event.button {
                PointerButton::Secondary => Some(
                    commands
                        .spawn_empty()
                        .add(SpawnPortalCmd {
                            parent_hex: click.target,
                        })
                        .id(),
                ),
                PointerButton::Primary => Some(
                    commands
                        .spawn_empty()
                        .add(SpawnTurretCmd {
                            parent_hex: click.target,
                        })
                        .id(),
                ),
                _ => None,
            };
            if let Some(_) = spawned_id {
                // mark the hex as not selected since something spawned on it
                materials.get_mut(material).unwrap().is_selected = 0.;
            }
        }
        // TODO: else, the hex is not constructible. Make it clear to the player!
    }
}

pub fn debug_display_non_constructible_hexes(
    grid: Res<HexGrid>,
    hexes: Query<(&Handle<HexMaterial>, Option<&NonConstructible>)>,
    mut materials: ResMut<Assets<HexMaterial>>,
    mut cached_hex_colors: Local<HashMap<Entity, Vec4>>,
) {
    for hex in grid.bounds.all_coords() {
        if let Some(entity) = grid.entities.get(&hex) {
            if let Ok((material, _)) = hexes.get(*entity) {
                let material = materials.get_mut(material).unwrap();
                if hexes
                    .get(*entity)
                    .is_ok_and(|(_, maybe_nonconstructible)| maybe_nonconstructible.is_some())
                {
                    cached_hex_colors.insert(*entity, material.color);
                    material.color.x = 1.0;
                    material.color.y = 0.0;
                    material.color.z = 0.0;
                    println!("Toggling unconstrible red ON {:?}", hex);
                } else if let Some(old_color) = cached_hex_colors.get(entity) {
                    println!("Toggling unconstrible red OFF {:?}", hex);
                    material.color = *old_color;
                }
            }
        }
    }
}
