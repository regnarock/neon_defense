mod image;
mod pipeline;

use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::RenderGraph,
        *,
    },
};
use bevy_mod_picking::prelude::*;

use crate::{actions::cursor::CursorScreenPos, enemy::SpawnEnemy, GameState};

use self::{
    image::BoardRenderImage,
    pipeline::{BoardShadersNode, BoardShadersPipeline},
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosOnBoard>()
            .add_systems(Startup, setup_board)
            // TODO: uncomment once self PR has been merged
            .add_systems(OnEnter(GameState::Playing), setup_board)
            .add_plugins(ExtractResourcePlugin::<BoardRenderImage>::default())
            .add_plugins(ExtractResourcePlugin::<CursorPosOnBoard>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<BoardShadersPipeline>()
            .add_systems(Render, pipeline::queue_bind_group.in_set(RenderSet::Queue));
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("board", BoardShadersNode::default());
        render_graph.add_node_edge("board", bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}

pub const BOARD_SIZE: (u32, u32) = (1200, 900);
pub const WORKGROUP_SIZE: u32 = 8;

pub fn setup_board(mut commands: Commands, images: ResMut<Assets<Image>>) {
    let render_image_res = BoardRenderImage::new(BOARD_SIZE.0, BOARD_SIZE.1, images);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(BOARD_SIZE.0 as f32, BOARD_SIZE.1 as f32)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            texture: render_image_res.0.clone(),
            ..default()
        },
        PickableBundle::default(),
        On::<Pointer<Move>>::run(update_pos_on_board),
        On::<Pointer<Click>>::run(spawn_enemy_on_click),
    ));

    info!("Board setup complete");
    commands.insert_resource(render_image_res);
}

pub fn spawn_enemy_on_click(
    mut commands: Commands,
    _click: Listener<Pointer<Click>>,
    mycoords: Res<CursorScreenPos>,
) {
    commands.add(SpawnEnemy {
        position: mycoords.0,
    });
}

#[derive(Resource, Default, ExtractResource, Clone)]
pub struct CursorPosOnBoard {
    pub position: Vec2,
    pub is_pressed: bool,
}

pub fn update_pos_on_board(
    event: Listener<Pointer<Move>>,
    mut pos_on_board: ResMut<CursorPosOnBoard>,
) {
    pos_on_board.position = event.pointer_location.position;
}
