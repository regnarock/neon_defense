use bevy::{
    prelude::*,
    render::{extract_resource::*, render_resource::*},
};

#[derive(Resource, Clone, Deref, ExtractResource, Debug, Default)]
pub struct BoardRenderImage(pub Handle<Image>);

impl BoardRenderImage {
    pub fn new(width: u32, height: u32, mut image_atlas: ResMut<Assets<Image>>) -> Self {
        BoardRenderImage(image_atlas.add(create_image(width, height)))
    }
}

fn create_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image
}
