use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph,
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
    },
};

use super::{image::BoardRenderImage, CursorPosOnBoard, BOARD_SIZE, WORKGROUP_SIZE};

#[derive(Resource)]
pub struct BoardShadersPipeline {
    compute: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}
impl FromWorld for BoardShadersPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Sand Simulation Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world.resource::<AssetServer>().load("shaders/board.wgsl");

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: shader.clone(),
            shader_defs: vec![],
            layout: vec![texture_bind_group_layout.clone()],
            entry_point: Cow::from("update"),
            push_constant_ranges: [PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<BoardPipelineParameters>() as u32,
            }]
            .to_vec(),
            label: Some(std::borrow::Cow::Borrowed("Board Pipeline")),
        });
        BoardShadersPipeline {
            compute: pipeline,
            texture_bind_group_layout,
        }
    }
}

#[derive(Resource, Debug)]
struct VoxelImageBindGroup(pub BindGroup);

pub fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<BoardShadersPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    voxels_image: Res<BoardRenderImage>,
    mut initiliazed: Local<bool>,
) {
    if *initiliazed {
        return;
    }
    let view = &gpu_images[&voxels_image.0];

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("Board Pipeline Bind Group"),
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    info!("[Resource inserted]{:?}", bind_group);
    commands.insert_resource(VoxelImageBindGroup(bind_group));
    *initiliazed = true;
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BoardPipelineParameters {
    size_x: f32,
    size_y: f32,
    mouse_x: f32,
    mouse_y: f32,
    mouse_pressed: f32,
    time: f32,
}

impl BoardPipelineParameters {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

pub enum BoardShadersState {
    Loading,
    Update,
}

pub struct BoardShadersNode {
    state: BoardShadersState,
}

impl Default for BoardShadersNode {
    fn default() -> Self {
        Self {
            state: BoardShadersState::Loading,
        }
    }
}

impl render_graph::Node for BoardShadersNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<BoardShadersPipeline>();

        // if the corresponding pipeline has loaded, transition to the next stage
        if let BoardShadersState::Loading = self.state {
            if let CachedPipelineState::Ok(_) = world
                .resource::<PipelineCache>()
                .get_compute_pipeline_state(pipeline.compute)
            {
                self.state = BoardShadersState::Update;
            }
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if let Some(texture_bind_group) = &world.get_resource::<VoxelImageBindGroup>() {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<BoardShadersPipeline>();
            let cursor = world.resource::<CursorPosOnBoard>();
            let time = world.resource::<Time>().elapsed_seconds();
            //let events = world.resource::<Events<Pointer<Click>>>();

            //events.get_reader();
            //let cursor_pos = world.get_resource::<CursorScreenPos>().unwrap();

            match self.state {
                BoardShadersState::Loading => {}
                BoardShadersState::Update => {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline.compute) {
                        pass.set_pipeline(pipeline);
                        pass.set_bind_group(0, &texture_bind_group.0, &[]);
                        let param = BoardPipelineParameters {
                            size_x: BOARD_SIZE.0 as f32,
                            size_y: BOARD_SIZE.1 as f32,
                            mouse_x: cursor.position.x,
                            mouse_y: cursor.position.y,
                            mouse_pressed: 1.,
                            time,
                        };

                        pass.set_push_constants(0, param.as_bytes());
                        pass.dispatch_workgroups(
                            BOARD_SIZE.0 / WORKGROUP_SIZE,
                            BOARD_SIZE.1 / WORKGROUP_SIZE,
                            1,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
