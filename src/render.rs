use crate::{pipeline::PointCloudPipeline, PointCloudAsset};
use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAsset,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferBinding,
            BufferInitDescriptor, BufferUsages,
        },
        renderer::RenderDevice,
    },
};

#[derive(Component, Clone)]
pub struct PotreePointCloud {
    pub mesh: Handle<PointCloudAsset>,
}

impl ExtractComponent for PotreePointCloud {
    type Query = &'static Self;

    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<'_, Self::Query>) -> Self {
        item.clone()
    }
}

pub struct PreparedPointCloudAsset {
    pub buffer: Buffer,
    pub num_points: u32,
    pub bind_group: BindGroup,
}

impl RenderAsset for PointCloudAsset {
    type ExtractedAsset = Self;

    type PreparedAsset = PreparedPointCloudAsset;

    type Param = (SRes<RenderDevice>, SRes<PointCloudPipeline>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        println!("Extracted asset");
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        println!("Prepared asset");
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::STORAGE,
            label: Some("Point cloud vertex buffer"),
            contents: extracted_asset.mesh.get_vertex_buffer_data().as_slice(),
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "point cloud buffer bind group".into(),
            layout: &pipeline.entity_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: bevy::render::render_resource::BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        Ok(PreparedPointCloudAsset {
            buffer,
            num_points: extracted_asset.mesh.count_vertices() as u32,
            bind_group,
        })
    }
}
