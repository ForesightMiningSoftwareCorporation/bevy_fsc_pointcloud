use crate::PointCloudPipelineKey;
use crate::{pipeline::PointCloudPipeline, PointCloudAsset};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BufferDescriptor, CachedRenderPipelineId, PipelineCache, SpecializedRenderPipelines,
};
use bevy::render::view::VisibleEntities;
use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        render_asset::RenderAsset,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferInitDescriptor,
            BufferUsages, ShaderType,
        },
        renderer::RenderDevice,
        Extract,
    },
};
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

#[derive(Component)]
pub struct PointCloudDrawList {
    pub list: Vec<PointCloudDrawData>,
}

pub struct PointCloudDrawData {
    pub entity: Entity,
    pub pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn queue_point_cloud(
    pipeline: Res<PointCloudPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PointCloudPipeline>>,
    cache: Res<PipelineCache>,
    views: Query<(Entity, &VisibleEntities)>,
    items: Query<&Handle<PointCloudAsset>>,
    point_clouds: Res<RenderAssets<PointCloudAsset>>,
    msaa: Option<Res<Msaa>>,
    mut commands: Commands,
) {
    let msaa = msaa.map(|a| a.samples()).unwrap_or(1);
    for (view_entity, entities) in &views {
        let mut list = vec![];
        for &entity in &entities.entities {
            if let Some(asset) = items
                .get(entity)
                .ok()
                .and_then(|handle| point_clouds.get(handle))
            {
                let key = PointCloudPipelineKey {
                    colored: asset.colored,
                    animated: asset.animation_buffer.is_some(),
                    msaa,
                };

                let pipeline_id = pipelines.specialize(&cache, &pipeline, key);
                list.push(PointCloudDrawData {
                    entity,
                    pipeline_id,
                });
            }
        }
        if !list.is_empty() {
            commands
                .entity(view_entity)
                .insert(PointCloudDrawList { list });
        }
    }
}

pub struct PreparedPointCloudAsset {
    pub buffer: Buffer,
    pub num_points: u32,
    pub bind_group: Option<BindGroup>,

    pub animation_buffer: Option<(Buffer, Buffer)>,
    pub frames: Option<Frames>,
    pub current_animation_frame: usize,
    pub animation_time: f32,
    pub animation_frame_start_time: f32,
    pub animation_scale: Vec3,

    pub colored: bool,
}

impl PreparedPointCloudAsset {
    pub fn update_bind_group(
        &mut self,
        render_device: &RenderDevice,
        pipeline: &PointCloudPipeline,
    ) {
        let mut bind_group_entires = vec![BindGroupEntry {
            binding: 0,
            resource: self.buffer.as_entire_binding(),
        }];
        if let Some((animation_buffer, next)) = self.animation_buffer.as_ref() {
            bind_group_entires.push(BindGroupEntry {
                binding: 1,
                resource: animation_buffer.as_entire_binding(),
            });
            bind_group_entires.push(BindGroupEntry {
                binding: 2,
                resource: next.as_entire_binding(),
            });
        }
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "point cloud buffer bind group".into(),
            layout: if self.animation_buffer.is_some() {
                &pipeline.animated_entity_layout
            } else {
                &pipeline.entity_layout
            },
            entries: &bind_group_entires,
        });
        self.bind_group = Some(bind_group);
    }
}

impl RenderAsset for PointCloudAsset {
    type ExtractedAsset = Self;

    type PreparedAsset = PreparedPointCloudAsset;

    type Param = (SRes<RenderDevice>, SRes<PointCloudPipeline>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::STORAGE,
            label: Some("Point cloud vertex buffer"),
            contents: extracted_asset.mesh.get_vertex_buffer_data().as_slice(),
        });

        let animation_buffer = if extracted_asset.animation.is_some() {
            let size = extracted_asset
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .len() as u64
                * std::mem::size_of::<f32>() as u64
                * 3
                + std::mem::size_of::<f32>() as u64;
            let animation_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("AnimationBuffer"),
                size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let animation_buffer_next = render_device.create_buffer(&BufferDescriptor {
                label: Some("AnimationBufferNext"),
                size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            Some((animation_buffer, animation_buffer_next))
        } else {
            None
        };
        let mut asset = PreparedPointCloudAsset {
            buffer,
            num_points: extracted_asset.mesh.count_vertices() as u32,
            bind_group: None,
            animation_buffer,
            frames: extracted_asset.animation,
            current_animation_frame: 0,
            animation_time: 0.0,
            animation_frame_start_time: 0.0,
            animation_scale: extracted_asset.animation_scale,
            colored: extracted_asset
                .mesh
                .contains_attribute(Mesh::ATTRIBUTE_COLOR),
        };
        asset.update_bind_group(render_device, pipeline);
        Ok(asset)
    }
}
