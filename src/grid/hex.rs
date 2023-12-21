use bevy::{
    ecs::system::EntityCommand,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle},
};
use bevy_eventlistener::{
    callbacks::{Listener, ListenerInput},
    event_listener::On,
};
use bevy_mod_picking::{
    events::{Click, Out, Over, Pointer},
    pointer::PointerButton,
};
use hexx::Hex;

use crate::{enemies::enemy::SpawnEnemy, enemies::portals::SpawnPortal, turret::SpawnTurret};

use super::HexGrid;

#[derive(Debug, Default, Component)]
pub struct HexCell {
    pub dist: u32,
    pub content: Option<Entity>,
}

#[derive(Debug, Default, Component)]
pub struct NonConstructible;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HexMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub is_selected: f32,
}

impl Material2d for HexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hex.wgsl".into()
    }
}

pub struct SpawnHex {
    pub position: Vec2,
    pub hex: Hex,
    pub mesh: Handle<Mesh>,
}

impl EntityCommand for SpawnHex {
    fn apply(self, id: Entity, world: &mut World) {
        let color = Color::rgb(5.0, 0., 0.);
        let asset = HexMaterial {
            color: color.into(),
            is_selected: 0.,
        };
        let material = world.resource_scope(
            |_world: &mut World, mut materials: Mut<Assets<HexMaterial>>| materials.add(asset),
        );
        world.entity_mut(id).insert((
            MaterialMesh2dBundle {
                mesh: self.mesh.clone().into(),
                material,
                transform: Transform::from_xyz(self.position.x, self.position.y, -1.0),
                ..default()
            },
            HexCell {
                dist: 0,
                content: None,
            },
            On::<Pointer<Over>>::run(select_hex),
            On::<Pointer<Out>>::run(deselect_hex),
            On::<Pointer<Click>>::send_event::<HexClicked>(),
        ));
    }
}

pub struct UpdateHexContent {
    pub content: Entity,
}

impl EntityCommand for UpdateHexContent {
    fn apply(self, id: Entity, world: &mut World) {
        let mut entity_mut = world.entity_mut(id);
        let mut cell = entity_mut.get_mut::<HexCell>().unwrap();

        cell.content = Some(self.content);
        cell.dist = u32::MAX;
    }
}

pub fn select_hex(
    event: Listener<Pointer<Over>>,
    mut hexes: Query<&Handle<HexMaterial>>,
    mut materials: ResMut<Assets<HexMaterial>>,
) {
    if let Ok(hex_material) = hexes.get_mut(event.target) {
        materials.get_mut(hex_material).unwrap().is_selected = 1.;
    }
}

pub fn deselect_hex(
    event: Listener<Pointer<Out>>,
    mut hexes: Query<&Handle<HexMaterial>>,
    mut materials: ResMut<Assets<HexMaterial>>,
) {
    if let Ok(hex_material) = hexes.get_mut(event.target) {
        materials.get_mut(hex_material).unwrap().is_selected = 0.;
    }
}

#[derive(Event)]
pub struct HexClicked {
    pub event: Click,
    pub target: Entity,
}

impl From<ListenerInput<Pointer<Click>>> for HexClicked {
    fn from(value: ListenerInput<Pointer<Click>>) -> Self {
        HexClicked {
            event: value.event.clone(),
            target: value.target,
        }
    }
}

pub fn on_click(
    mut commands: Commands,
    mut clicks: EventReader<HexClicked>,
    hexes: Query<&Transform, Without<NonConstructible>>,
    _grid: Res<HexGrid>,
) {
    for click in clicks.read() {
        if let Ok(transform) = hexes.get(click.target) {
            match click.event.button {
                PointerButton::Secondary => {
                    // TODO: allow configuration of different portals (portals/enemies inventory)
                    commands.add(SpawnPortal {
                        position: transform.translation.xy(),
                        spawn_n_times: 10,
                        delay_s: 3.0,
                        enemy_def: crate::enemies::enemy::EnemyDef::Ship01,
                    });
                }
                PointerButton::Primary => {
                    let turret_id = commands
                        .spawn_empty()
                        .add(SpawnTurret {
                            position: transform.translation.xy(),
                            at_hex: click.target,
                        })
                        .id();
                    commands
                        .entity(click.target)
                        .add(UpdateHexContent { content: turret_id });
                }
                _ => {}
            }
        }
        // TODO: UX: else, means the hex is not constructible. Make it clear to player.
    }
}
