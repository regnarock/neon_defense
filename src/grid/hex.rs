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
use bevy_mod_picking::events::{Click, Out, Over, Pointer};
use hexx::Hex;

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

pub struct SpawnHexCmd {
    pub position: Vec2,
    pub hex: Hex,
    pub mesh: Handle<Mesh>,
}

impl EntityCommand for SpawnHexCmd {
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
            HexCell { dist: 0 },
            On::<Pointer<Over>>::run(select_hex),
            On::<Pointer<Out>>::run(deselect_hex),
            On::<Pointer<Click>>::send_event::<HexClicked>(),
        ));
    }
}

pub fn reset_dist_on_content_change() {}

pub fn select_hex(
    event: Listener<Pointer<Over>>,
    mut hexes: Query<(&Handle<HexMaterial>, Option<&Children>), With<HexCell>>,
    mut materials: ResMut<Assets<HexMaterial>>,
) {
    if let Ok((material, content)) = hexes.get_mut(event.target) {
        // don't select if the hex right under the cursor is occupied
        if content.is_none() {
            materials.get_mut(material).unwrap().is_selected = 1.;
        }
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
