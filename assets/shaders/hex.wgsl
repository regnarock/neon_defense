#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct Material {
    color: vec4<f32>,
    is_hover: f32,
};

@group(1) @binding(0) var<uniform> material: Material;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    if (material.is_hover > 0.5) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
    return material.color;
}
