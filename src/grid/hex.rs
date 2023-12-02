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

use crate::{enemy::SpawnEnemy, turret::SpawnTurret};

use super::HexGrid;

#[derive(Debug, Default, Component)]
pub struct HexCell {
    pub dist: u32,
}

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
        world.resource_scope(
            |world: &mut World, mut materials: Mut<Assets<HexMaterial>>| {
                world.entity_mut(id).insert((
                    MaterialMesh2dBundle {
                        mesh: self.mesh.clone().into(),
                        material: materials.add(asset),
                        transform: Transform::from_xyz(self.position.x, self.position.y, -1.0),
                        ..default()
                    },
                    HexCell { dist: 0 },
                    On::<Pointer<Over>>::run(select_hex),
                    On::<Pointer<Out>>::run(deselect_hex),
                    On::<Pointer<Click>>::send_event::<SpawnOnClick>(),
                ));
            },
        );
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
pub struct SpawnOnClick {
    event: Click,
    target: Entity,
}

impl From<ListenerInput<Pointer<Click>>> for SpawnOnClick {
    fn from(value: ListenerInput<Pointer<Click>>) -> Self {
        SpawnOnClick {
            event: value.event.clone(),
            target: value.target,
        }
    }
}

pub fn spawn_on_click(
    mut commands: Commands,
    mut clicks: EventReader<SpawnOnClick>,
    hexes: Query<&Transform, &HexCell>,
    _grid: Res<HexGrid>,
) {
    for click in clicks.read() {
        match click.event.button {
            PointerButton::Secondary => {
                commands.add(SpawnEnemy {
                    position: hexes.get(click.target).unwrap().translation.xy(),
                });
            }
            PointerButton::Primary => {
                commands.add(SpawnTurret {
                    at_hex: click.target,
                });
            }
            _ => {}
        }
    }
}
