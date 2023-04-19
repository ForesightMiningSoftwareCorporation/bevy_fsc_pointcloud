mod clippling_planes;
#[cfg(feature = "las")]
mod las_loader;
#[cfg(feature = "opd")]
mod opd_loader;
mod pipeline;
mod render;
mod render_graph;
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_component::{UniformComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        render_asset::{PrepareAssetLabel, RenderAssetPlugin},
        render_graph::RenderGraph,
        render_resource::ShaderStage,
        RenderApp, RenderStage,
    },
};
pub use clippling_planes::{ClippingPlaneBundle, ClippingPlaneRange};
#[cfg(feature = "las")]
pub use las_loader::*;
#[cfg(feature = "opd")]
pub use opd_loader::*;
pub use pipeline::*;
pub use render::*;
pub use render_graph::*;

#[derive(Default)]
pub struct PointCloudPlugin {
    pub colored: bool,
    pub animated: bool,
}

impl Plugin for PointCloudPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let point_cloud_pipeline = PointCloudPipeline::from_app(app, self.colored, self.animated);
        app.add_asset::<PointCloudAsset>();

        #[cfg(feature = "las")]
        app.add_asset_loader(LasLoader);
        #[cfg(feature = "opd")]
        app.add_asset_loader(OpdLoader);
        app.add_plugin(
            RenderAssetPlugin::<PointCloudAsset>::with_prepare_asset_label(
                PrepareAssetLabel::AssetPrepare,
            ),
        )
        .add_plugin(UniformComponentPlugin::<PointCloudUniform>::default());
        if self.animated {
            app.init_resource::<PointCloudPlaybackControl>()
                .add_plugin(ExtractResourcePlugin::<PointCloudPlaybackControl>::default());
        }
        load_internal_asset!(app, POINT_CLOUD_VERT_SHADER_HANDLE, "shader.vert", |s| {
            Shader::from_glsl(s, ShaderStage::Vertex)
        });
        load_internal_asset!(app, POINT_CLOUD_FRAG_SHADER_HANDLE, "shader.frag", |s| {
            Shader::from_glsl(s, ShaderStage::Fragment)
        });
        load_internal_asset!(
            app,
            EYE_DOME_LIGHTING_SHADER_HANDLE,
            "eye-dome.wgsl",
            Shader::from_wgsl
        );
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_system_to_stage(RenderStage::Extract, extract_point_cloud)
            .add_system_to_stage(RenderStage::Queue, prepare_point_cloud_bind_group)
            .add_system_to_stage(RenderStage::Queue, prepare_view_targets)
            .add_system_to_stage(
                RenderStage::Extract,
                clippling_planes::extract_clipping_planes,
            )
            .add_system_to_stage(
                RenderStage::Prepare,
                clippling_planes::prepare_clipping_planes,
            )
            .init_resource::<clippling_planes::UniformBufferOfGpuClippingPlaneRanges>()
            .init_resource::<PointCloudBindGroup>()
            .insert_resource(point_cloud_pipeline);
        if self.animated {
            render_app
                .add_system_to_stage(RenderStage::Prepare, prepare_animated_assets)
                .init_resource::<PointCloudPlaybackControl>();
        }
        let point_cloud_node = PointCloudNode::new(&mut render_app.world);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_3d_graph = render_graph
            .get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME)
            .unwrap();

        draw_3d_graph.add_node(PointCloudNode::NAME, point_cloud_node);
        draw_3d_graph
            .add_node_edge(
                bevy::core_pipeline::core_3d::graph::node::MAIN_PASS,
                PointCloudNode::NAME,
            )
            .unwrap();
        draw_3d_graph
            .add_slot_edge(
                draw_3d_graph.input_node().unwrap().id,
                bevy::core_pipeline::core_3d::graph::input::VIEW_ENTITY,
                PointCloudNode::NAME,
                PointCloudNode::IN_VIEW,
            )
            .unwrap();
    }
}
