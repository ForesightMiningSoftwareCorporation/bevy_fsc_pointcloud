use crate::{pipeline::PointCloudPipeline, PointCloudAsset};
use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, ExtractComponent},
        render_asset::RenderAsset,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferBinding,
            BufferInitDescriptor, BufferUsages, ShaderType,
        },
        renderer::RenderDevice,
        Extract,
    },
};
use bevy::render::render_resource::BufferDescriptor;
use opd_parser::Frames;
#[derive(Component, Clone)]
pub struct PotreePointCloud {
    pub mesh: Handle<PointCloudAsset>,
    pub point_size: f32,
}
#[derive(Component, Clone, ShaderType)]
pub struct PointCloudUniform {
    pub transform: Mat4,
    pub point_size: f32,
}

pub(crate) fn extract_point_cloud(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, &PotreePointCloud, &GlobalTransform)>>,
) {
    let mut values = Vec::with_capacity(*previous_len);

    for (entity, point_cloud, transform) in query.iter() {
        values.push((
            entity,
            (
                PointCloudUniform {
                    transform: transform.compute_matrix(),
                    point_size: point_cloud.point_size,
                },
                point_cloud.mesh.clone(),
            ),
        ));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

pub struct PreparedPointCloudAsset {
    pub buffer: Buffer,
    pub num_points: u32,
    pub bind_group: BindGroup,

    pub animation_buffer: Option<Buffer>,
    pub frames: Option<Frames>,
    pub current_animation_frame: usize,
    pub animation_time: f32,
    pub animation_scale: Vec3
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

        let mut animation_buffer = if extracted_asset.animation.is_some(){
            Some(render_device.create_buffer(&BufferDescriptor {
                label: Some("AnimationBuffer"),
                size: extracted_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len() as u64 * std::mem::size_of::<f32>() as u64 * 3,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }))
        } else {
            None
        };

        let mut bind_group_entires = vec![
            BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding()
            }
        ];
        if let Some(animation_buffer) = animation_buffer.as_ref() {
            bind_group_entires.push(BindGroupEntry {
                binding: 1,
                resource: animation_buffer.as_entire_binding()
            });
        }
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "point cloud buffer bind group".into(),
            layout: &pipeline.entity_layout,
            entries:
                &bind_group_entires,
        });

        Ok(PreparedPointCloudAsset {
            buffer,
            num_points: extracted_asset.mesh.count_vertices() as u32,
            bind_group,
            animation_buffer,
            frames: extracted_asset.animation,
            current_animation_frame: 0,
            animation_time: 0.0,
            animation_scale: extracted_asset.animation_scale
        })
    }
}
