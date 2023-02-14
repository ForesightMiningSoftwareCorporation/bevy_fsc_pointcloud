mod loader;
mod pipeline;
mod render;
mod render_graph;

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_asset::{PrepareAssetLabel, RenderAssetPlugin},
        render_graph::RenderGraph,
        render_resource::ShaderStage,
        RenderApp, RenderStage,
    },
};
pub use loader::*;
pub use pipeline::*;
pub use render::*;
pub use render_graph::*;

#[derive(Default)]
pub struct PointCloudPlugin;

impl Plugin for PointCloudPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let point_cloud_pipeline = PointCloudPipeline::from_app(app);
        app.add_asset::<PointCloudAsset>()
            .add_asset_loader(LasLoader)
            //.add_plugin(LookTransformPlugin)
            //.add_plugin(FpsCameraPlugin::default())
            .add_plugin(
                RenderAssetPlugin::<PointCloudAsset>::with_prepare_asset_label(
                    PrepareAssetLabel::AssetPrepare,
                ),
            )
            .add_plugin(ExtractComponentPlugin::<PotreePointCloud>::default());

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
            .add_system_to_stage(RenderStage::Prepare, prepare_point_cloud_bind_group)
            .add_system_to_stage(RenderStage::Queue, prepare_view_targets)
            .init_resource::<PointCloudBindGroup>()
            .insert_resource(point_cloud_pipeline);
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
