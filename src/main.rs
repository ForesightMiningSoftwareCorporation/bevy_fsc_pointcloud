use bevy::{prelude::*, asset::{AssetLoader, LoadedAsset, Asset, load_internal_asset}, render::{view::{ExtractedView, VisibleEntities, ViewDepthTexture, ViewTarget, ViewUniforms, ViewUniformOffset}, render_phase::{RenderPhase, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass}, render_resource::{RenderPipelineId, Buffer, BufferUsages, BufferInitDescriptor, RenderPassDescriptor, Operations, LoadOp, RenderPassDepthStencilAttachment, RawVertexBufferLayout, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat, BindGroupDescriptor, BindGroupEntry, BindGroup, ShaderStage}, render_asset::{RenderAsset, RenderAssetPlugin, PrepareAssetLabel, RenderAssets}, render_graph::{SlotInfo, SlotType, RenderGraph}, camera::ExtractedCamera, extract_component::{ExtractComponent, ExtractComponentPlugin}, RenderApp, RenderStage, mesh::MeshVertexAttribute}, core_pipeline::{core_3d::{Opaque3d, MainPass3dNode}, clear_color::ClearColorConfig}, ecs::system::{lifetimeless::SRes, SystemParamItem}, core::cast_slice};
use las::Read;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_resource::{SamplerBindingType, TextureSampleType, TextureViewDimension};
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, BufferSize, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, FragmentState,
            FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            RenderPipelineDescriptor, ShaderStages, ShaderType, StencilFaceState, StencilState,
            TextureFormat, VertexState,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewUniform,
    },
};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

struct LasLoader;

#[repr(transparent)]
struct Point {
    inner: [f32; 3]
}
impl From<[f32; 3]> for Point {
    fn from(inner: [f32; 3]) -> Self {
        Self { inner }
    }
}
impl Point {
    pub fn min(&self, other: &Self) -> Self {
        Point{inner: [
            self.inner[0].min(other.inner[0]),
            self.inner[1].min(other.inner[1]),
            self.inner[2].min(other.inner[2]),
        ]}
    }
    pub fn max(&self, other: &Self) -> Self {
        Point{inner: [
            self.inner[0].max(other.inner[0]),
            self.inner[1].max(other.inner[1]),
            self.inner[2].max(other.inner[2]),
        ]}
    }
}

pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Float32x3);
impl AssetLoader for LasLoader {
    fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let mut reader = las::Reader::new(std::io::Cursor::new(bytes)).expect("Unable to open reader");
            println!("headers: {:?}", reader.header());
            let mut mesh = Mesh::new(PrimitiveTopology::PointList);
            let mut max: Point = [f32::MIN; 3].into();
            let mut min: Point = [f32::MAX; 3].into();
            let mut positions = reader.points().take(500000).map(|a| {
                let p = a.unwrap();
                let p: Point = [p.x as f32, p.y as f32, p.z as f32].into();
                min = min.min(&p);
                max = max.max(&p);
                p.inner
            }).collect::<Vec<_>>();

            let colors = reader.points().take(500000).map(|a| {
                let p = a.unwrap();
                let intensity = p.intensity as f32 * 0.001;
                [intensity, intensity, intensity]
            }).collect::<Vec<_>>();

            let aabb = [max.inner[0] - min.inner[0], max.inner[1] - min.inner[1], max.inner[2] - min.inner[2]];
            let scale = aabb[0].max(aabb[1]).max(aabb[2]);
            for i in positions.iter_mut() {
                i[0] -= min.inner[0];
                i[1] -= min.inner[1];
                i[2] -= min.inner[2];
                i[0] /= scale;
                i[1] /= scale;
                i[2] /= scale;
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(ATTRIBUTE_COLOR, colors);
            println!("Loaded asset, max {:?}, min {:?}", max.inner, min.inner);
            let asset = PointCloudAsset { mesh };
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }
    fn extensions(&self) -> &[&str] {
        &["las", "laz"]
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "806a9a3b-04db-4e4e-b509-ab35ef3a6c43"]
struct PointCloudAsset {
    mesh: Mesh
}



#[derive(Component, Clone)]
struct PotreePointCloud {
    mesh: Handle<PointCloudAsset>,
}

impl ExtractComponent for PotreePointCloud {
    type Query = &'static Self;

    type Filter = ();

    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}

struct PreparedPointCloudAsset {
    buffer: Buffer,
    num_points: u32,
}

impl RenderAsset for PointCloudAsset {
    type ExtractedAsset = Self;

    type PreparedAsset = PreparedPointCloudAsset;

    type Param = SRes<RenderDevice>;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        println!("Extracted asset");
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        render_device: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>> {
        println!("Prepared asset");
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("Point cloud vertex buffer"),
            contents: extracted_asset.mesh.get_vertex_buffer_data().as_slice()
        });
        Ok(PreparedPointCloudAsset { buffer, num_points: extracted_asset.mesh.count_vertices() as u32 })
    }
}

fn main() {
    let mut app = App::new();

    app
    .add_plugins(DefaultPlugins)
    .add_asset::<PointCloudAsset>()
    .add_asset_loader(LasLoader)
    .add_startup_system(startup)
    .add_plugin(LookTransformPlugin)
    .add_plugin(FpsCameraPlugin::default())
    .add_plugin(RenderAssetPlugin::<PointCloudAsset>::with_prepare_asset_label(PrepareAssetLabel::AssetPrepare))
    .add_plugin(ExtractComponentPlugin::<PotreePointCloud>::default());

    load_internal_asset!(app, POINT_CLOUD_VERT_SHADER_HANDLE, "shader.vert", |s| Shader::from_glsl(s, ShaderStage::Vertex));
    load_internal_asset!(app, POINT_CLOUD_FRAG_SHADER_HANDLE, "shader.frag", |s| Shader::from_glsl(s, ShaderStage::Fragment));
    let render_app = app.sub_app_mut(RenderApp);


    render_app
    .add_system_to_stage(RenderStage::Prepare, prepare_point_cloud_bind_group)
    .init_resource::<PointCloudBindGroup>()
    .init_resource::<PointCloudPipeline>();
    let point_cloud_node = PointCloudNode::new(&mut render_app.world);



    let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
    let draw_3d_graph = render_graph.get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME).unwrap();
    
    draw_3d_graph.add_node(PointCloudNode::NAME, point_cloud_node);
    draw_3d_graph.add_node_edge(bevy::core_pipeline::core_3d::graph::node::MAIN_PASS, PointCloudNode::NAME);
    draw_3d_graph.add_slot_edge(
        draw_3d_graph.input_node().id,
        bevy::core_pipeline::core_3d::graph::input::VIEW_ENTITY,
        PointCloudNode::NAME,
        PointCloudNode::IN_VIEW,
    );
    app.run();

}


fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let path = std::env::args()
    .skip(1)
    .next();
    let mesh: Handle<PointCloudAsset> = asset_server.load("points.las");

    commands.spawn(SpatialBundle::default())
    .insert(PotreePointCloud {
        mesh,
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-20.0, 20.5, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })        .insert(FpsCameraBundle::new(
        FpsCameraController {
            translate_sensitivity: 2.0,
            ..Default::default()
        },
        Vec3::new(5.0, 2.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
    ));
}

pub struct PointCloudNode {
    query: QueryState<
        (
            &'static ExtractedCamera,
            &'static ExtractedView,
            &'static Camera3d,
            &'static ViewTarget,
            &'static ViewDepthTexture,
            &'static ViewUniformOffset
        ),
        With<ExtractedView>,
    >,
    entity_query: QueryState<
        (
            &'static PotreePointCloud,
        ),
    >
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

use bevy::render::render_graph::Node;
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
        let (camera, view, camera_3d, target, depth, view_uniform_offset) =
            match self.query.get_manual(world, view_entity) {
                Ok(query) => query,
                Err(_) => {
                    return Ok(());
                } // No window
            };
        let mut render_pass = render_context
            .command_encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("point_cloud"),
                // NOTE: The opaque pass loads the color
                // buffer as well as writing to it.
                color_attachments: &[Some(target.get_color_attachment(Operations {
                    load: LoadOp::Load,
                    store: true,
                }))],
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
        
        let point_cloud_pipeline = world.resource::<PointCloudPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.get_render_pipeline(point_cloud_pipeline.pipeline_id);
        if pipeline.is_none() {
            println!("No pipeline");
            return Ok(());
        }
        let pipeline = pipeline.unwrap();
        render_pass.set_pipeline(pipeline);
        let bind_groups = world.resource::<PointCloudBindGroup>();
        if bind_groups.bind_group.is_none() {
            println!("No bind group");
            return Ok(());
        }
        render_pass.set_bind_group(0, &bind_groups.bind_group.as_ref().unwrap(), &[view_uniform_offset.offset]);
        //render_pass.set_push_constants(ShaderStages::VERTEX | ShaderStages::FRAGMENT, 0, data)

        let render_assets = world.resource::<RenderAssets<PointCloudAsset>>();
        for (point_cloud, ) in self.entity_query.iter_manual(&world) {
            let point_cloud_asset = render_assets.get(&point_cloud.mesh);
            if point_cloud_asset.is_none() {
                continue;
            }
            let point_cloud_asset = point_cloud_asset.unwrap();
            render_pass.set_vertex_buffer(0, *point_cloud_asset.buffer.slice(..));
            render_pass.draw(0..point_cloud_asset.num_points, 0..1);
        }

        Ok(())
    }

    fn output(&self) -> Vec<SlotInfo> {
        Vec::new()
    }
}


pub(crate) const POINT_CLOUD_VERT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf01);
pub(crate) const POINT_CLOUD_FRAG_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf02);


#[derive(Resource)]
pub struct PointCloudPipeline {
    pub pipeline_id: CachedRenderPipelineId,
    pub view_layout: BindGroupLayout,
}


#[derive(Resource, Default)]
pub struct PointCloudBindGroup {
    bind_group: Option<BindGroup>,
}
fn prepare_point_cloud_bind_group(
    render_device: Res<RenderDevice>,
    pipeline: Res<PointCloudPipeline>,
    view_uniform: Res<ViewUniforms>,
    mut bind_groups: ResMut<PointCloudBindGroup>
) {
    if let Some(resource) = view_uniform.uniforms.binding() {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("point_cloud_bind_group"),
            layout: &pipeline.view_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource
                }
            ],
        });
        bind_groups.bind_group = Some(bind_group);
    }
}

impl FromWorld for PointCloudPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("PointCloudViewLabel"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: None },
                    count: None
                }
            ],
        });
        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("point_cloud_pipeline".into()),
            layout: Some(vec![
                view_layout.clone()
            ]),
            vertex: VertexState {
                shader: POINT_CLOUD_VERT_SHADER_HANDLE.typed(),
                shader_defs: Default::default(),
                entry_point: "main".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: 24,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }, VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    }],
                }
                ],
            },
            fragment: Some(FragmentState {
                shader: POINT_CLOUD_FRAG_SHADER_HANDLE.typed(),
                shader_defs: Default::default(),
                entry_point: "main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Point,
                conservative: false,
                topology: PrimitiveTopology::PointList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(pipeline_descriptor);
        println!("Pipeline queued");

        let layout = 
        Self {
            pipeline_id,
            view_layout
        };
        layout
    }
}

