use crate::pipeline::{EyeDomeViewTarget, PointCloudBindGroup, PointCloudPipeline};
use crate::{PointCloudAsset, PointCloudUniform};
use bevy::core_pipeline::core_3d::MainPass3dNode;
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, SlotInfo, SlotType};
use bevy::render::render_resource::{
    LoadOp, Operations, PipelineCache, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, ShaderStages,
};
use bevy::render::view::{
    ExtractedView, ViewDepthTexture, ViewTarget, ViewUniformOffset, VisibleEntities,
};

pub struct PointCloudNode {
    #[allow(clippy::type_complexity)]
    query: QueryState<
        (
            &'static ExtractedView,
            &'static ExtractedCamera,
            &'static ViewTarget,
            &'static ViewDepthTexture,
            &'static ViewUniformOffset,
            &'static EyeDomeViewTarget,
            &'static VisibleEntities,
        ),
        With<ExtractedView>,
    >,
    entity_query: QueryState<(
        Entity,
        &'static Handle<PointCloudAsset>,
        &'static DynamicUniformIndex<PointCloudUniform>,
    )>,
}

impl PointCloudNode {
    pub const NAME: &'static str = "point_cloud_node";
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
            entity_query: world.query_filtered(),
        }
    }
}

impl Node for PointCloudNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(MainPass3dNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.entity_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let point_cloud_pipeline = world.resource::<PointCloudPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_assets = world.resource::<RenderAssets<PointCloudAsset>>();
        let (
            view,
            camera,
            target,
            depth,
            view_uniform_offset,
            eye_dome_view_target,
            visible_entities,
        ) = match self.query.get_manual(world, view_entity) {
            Ok(query) => query,
            Err(_) => {
                return Ok(());
            } // No window
        };

        let mut tracked_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("point_cloud"),
            // NOTE: The opaque pass loads the color
            // buffer as well as writing to it.
            color_attachments: &[
                Some(target.get_color_attachment(Operations {
                    load: LoadOp::Load,
                    store: true,
                })),
                Some(RenderPassColorAttachment {
                    view: &eye_dome_view_target.depth_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK.into()),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth.view,
                // NOTE: The opaque main pass loads the depth buffer and possibly overwrites it
                depth_ops: Some(Operations {
                    // NOTE: 0.0 is the far plane due to bevy's use of reverse-z projections.
                    load: LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        if let Some(viewport) = camera.viewport.as_ref() {
            tracked_pass.set_camera_viewport(viewport);
        }
        let pipeline = pipeline_cache.get_render_pipeline(point_cloud_pipeline.pipeline_id);
        let eye_dome_pipeline =
            pipeline_cache.get_render_pipeline(point_cloud_pipeline.eye_dome_pipeline_id);
        if pipeline.is_none() || eye_dome_pipeline.is_none() {
            warn!("No pipeline");
            return Ok(());
        }
        let pipeline = pipeline.unwrap();
        let eye_dome_pipeline = eye_dome_pipeline.unwrap();

        tracked_pass.set_render_pipeline(pipeline);
        let bind_groups = world.resource::<PointCloudBindGroup>();
        if bind_groups.bind_group.is_none() || bind_groups.model_bind_group.is_none() {
            return Ok(());
        }
        tracked_pass.set_bind_group(
            0,
            bind_groups.bind_group.as_ref().unwrap(),
            &[view_uniform_offset.offset],
        );
        tracked_pass.set_vertex_buffer(0, point_cloud_pipeline.instanced_point_quad.slice(0..32));
        for (entity, point_cloud_asset, dynamic_index) in self.entity_query.iter_manual(world) {
            if !visible_entities.entities.contains(&entity) {
                continue;
            }
            let point_cloud_asset = render_assets.get(point_cloud_asset);
            if point_cloud_asset.is_none() {
                continue;
            }
            let point_cloud_asset = point_cloud_asset.unwrap();
            tracked_pass.set_bind_group(1, point_cloud_asset.bind_group.as_ref().unwrap(), &[]);
            tracked_pass.set_bind_group(
                2,
                bind_groups.model_bind_group.as_ref().unwrap(),
                &[dynamic_index.index()],
            );
            tracked_pass.draw(0..4, 0..point_cloud_asset.num_points);
        }
        drop(tracked_pass);

        let mut tracked_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("eye_dome_lighting"),
            // NOTE: The opaque pass loads the color
            // buffer as well as writing to it.
            color_attachments: &[Some(target.get_color_attachment(Operations {
                load: LoadOp::Load,
                store: true,
            }))],
            depth_stencil_attachment: None,
        });
        if let Some(viewport) = camera.viewport.as_ref() {
            tracked_pass.set_camera_viewport(viewport);
        }
        tracked_pass.set_render_pipeline(eye_dome_pipeline);

        let edl_strength: f32 = if view.projection.z_axis.w == -1.0 {
            // perspective projection
            // See https://github.com/bitshifter/glam-rs/blob/a35030d130c0464cbb07d6404df6843240182803/src/f32/scalar/mat4.rs#L843
            1.0
        } else {
            // orthographic projection
            // See https://github.com/bitshifter/glam-rs/blob/a35030d130c0464cbb07d6404df6843240182803/src/f32/scalar/mat4.rs#L924
            1.0 / view.projection.z_axis.z // near - far
        };

        tracked_pass.set_push_constants(ShaderStages::FRAGMENT, 0, unsafe {
            std::slice::from_raw_parts(&edl_strength as *const f32 as *const u8, 4)
        });
        tracked_pass.set_bind_group(0, &eye_dome_view_target.bind_group, &[]);
        tracked_pass.set_vertex_buffer(0, point_cloud_pipeline.instanced_point_quad.slice(0..32));
        tracked_pass.draw(0..4, 0..1);
        Ok(())
    }

    fn output(&self) -> Vec<SlotInfo> {
        Vec::new()
    }
}
