use bevy::{prelude::*, asset::{AssetLoader, LoadedAsset, Asset, load_internal_asset}, render::{view::{ExtractedView, VisibleEntities, ViewDepthTexture, ViewTarget, ViewUniforms, ViewUniformOffset}, render_phase::{RenderPhase, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass}, render_resource::{RenderPipelineId, Buffer, BufferUsages, BufferInitDescriptor, RenderPassDescriptor, Operations, LoadOp, RenderPassDepthStencilAttachment, RawVertexBufferLayout, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat, BindGroupDescriptor, BindGroupEntry, BindGroup, ShaderStage, CachedComputePipelineId, ComputePipelineDescriptor, StorageTextureAccess, Texture, TextureView, TextureDescriptor, TextureDimension, TextureUsages, Extent3d, AsBindGroupShaderType, RenderPassColorAttachment, ComputePassDescriptor, Sampler, SamplerDescriptor, BufferBinding}, render_asset::{RenderAsset, RenderAssetPlugin, PrepareAssetLabel, RenderAssets}, render_graph::{SlotInfo, SlotType, RenderGraph}, camera::ExtractedCamera, extract_component::{ExtractComponent, ExtractComponentPlugin}, RenderApp, RenderStage, mesh::MeshVertexAttribute, texture::TextureCache}, core_pipeline::{core_3d::{Opaque3d, MainPass3dNode}, clear_color::ClearColorConfig}, ecs::system::{lifetimeless::SRes, SystemParamItem}, core::cast_slice, utils::HashMap, window::PresentMode};
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
MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Float32);
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
                intensity
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
    bind_group: BindGroup,
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
    ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>> {
        println!("Prepared asset");
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::STORAGE,
            label: Some("Point cloud vertex buffer"),
            contents: extracted_asset.mesh.get_vertex_buffer_data().as_slice()
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor{
            label: "point cloud buffer bind group".into(), layout: &pipeline.entity_layout, entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: bevy::render::render_resource::BindingResource::Buffer(BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None
                    })
                }
            ]
        }
        );
        Ok(PreparedPointCloudAsset { buffer, num_points: extracted_asset.mesh.count_vertices() as u32, bind_group })
    }
}

fn main() {
    let mut app = App::new();
    // 192 fps
    app
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            present_mode: PresentMode::Immediate,
            ..Default::default()
        },
        ..Default::default()
    }))
    .add_asset::<PointCloudAsset>()
    .add_asset_loader(LasLoader)
    .add_startup_system(startup)
    //.add_plugin(LookTransformPlugin)
    //.add_plugin(FpsCameraPlugin::default())
    .add_plugin(RenderAssetPlugin::<PointCloudAsset>::with_prepare_asset_label(PrepareAssetLabel::AssetPrepare))
    .add_plugin(ExtractComponentPlugin::<PotreePointCloud>::default());

    load_internal_asset!(app, POINT_CLOUD_VERT_SHADER_HANDLE, "shader.vert", |s| Shader::from_glsl(s, ShaderStage::Vertex));
    load_internal_asset!(app, POINT_CLOUD_FRAG_SHADER_HANDLE, "shader.frag", |s| Shader::from_glsl(s, ShaderStage::Fragment));
    load_internal_asset!(app, EYE_DOME_LIGHTING_SHADER_HANDLE, "eye-dome.wgsl", Shader::from_wgsl);
    let render_app = app.sub_app_mut(RenderApp);


    render_app
    .add_system_to_stage(RenderStage::Prepare, prepare_point_cloud_bind_group)
    .add_system_to_stage(RenderStage::Queue, prepare_view_targets)
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
        transform: Transform::from_xyz(1.0, 1.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

pub struct PointCloudNode {
    query: QueryState<
        (
            &'static ExtractedView,
            &'static ViewTarget,
            &'static ViewDepthTexture,
            &'static ViewUniformOffset,
            &'static EyeDomeViewTarget,
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
        let (view, target, depth, view_uniform_offset, eye_dome_view_target) =
            match self.query.get_manual(world, view_entity) {
                Ok(query) => query,
                Err(_) => {
                    return Ok(());
                } // No window
            };
        let mut color = Color::rgba(0.0, 0.0, 0.0, 0.0);
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
                    view: &eye_dome_view_target.depth_texture_view,
                    // NOTE: The opaque main pass loads the depth buffer and possibly overwrites it
                    depth_ops: Some(Operations {
                        // NOTE: 0.0 is the far plane due to bevy's use of reverse-z projections.
                        load: LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        
        let point_cloud_pipeline = world.resource::<PointCloudPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.get_render_pipeline(point_cloud_pipeline.pipeline_id);
        let eye_dome_pipeline = pipeline_cache.get_compute_pipeline(point_cloud_pipeline.eye_dome_pipeline_id);
        if pipeline.is_none() || eye_dome_pipeline.is_none() {
            println!("No pipeline");
            return Ok(());
        }
        let pipeline = pipeline.unwrap();
        let eye_dome_pipeline = eye_dome_pipeline.unwrap();

        render_pass.set_pipeline(pipeline);
        let bind_groups = world.resource::<PointCloudBindGroup>();
        if bind_groups.bind_group.is_none() {
            println!("No bind group");
            return Ok(());
        }
        render_pass.set_bind_group(0, &bind_groups.bind_group.as_ref().unwrap(), &[view_uniform_offset.offset]);
        render_pass.set_vertex_buffer(0, *point_cloud_pipeline.instanced_point_quad.slice(0..32));
        let render_assets = world.resource::<RenderAssets<PointCloudAsset>>();
        for (point_cloud, ) in self.entity_query.iter_manual(&world) {
            let point_cloud_asset = render_assets.get(&point_cloud.mesh);
            if point_cloud_asset.is_none() {
                continue;
            }
            let point_cloud_asset = point_cloud_asset.unwrap();
            render_pass.set_bind_group(1, &point_cloud_asset.bind_group, &[]);

            render_pass.draw(0..4, 0..point_cloud_asset.num_points);
        }

        drop(render_pass);
        let mut render_pass = render_context.command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: "Eye Dome Lighting".into()
        });
        render_pass.set_pipeline(eye_dome_pipeline);
        render_pass.set_bind_group(0, &eye_dome_view_target.bind_group, &[]);
        render_pass.dispatch_workgroups(view.viewport.z / 8, view.viewport.w / 8, 1);

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
    pub(crate) const EYE_DOME_LIGHTING_SHADER_HANDLE: HandleUntyped =
        HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf03);


#[derive(Resource)]
pub struct PointCloudPipeline {
    pub pipeline_id: CachedRenderPipelineId,
    pub view_layout: BindGroupLayout,
    pub entity_layout: BindGroupLayout,

    pub eye_dome_pipeline_id: CachedComputePipelineId,
    pub eye_dome_image_layout: BindGroupLayout,

    pub sampler: Sampler,
    pub instanced_point_quad: Buffer,
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

const QUAD_VERTEX_BUF: &'static [f32] = &[
    0.0, 1.0,
    0.0, 0.0,
    1.0, 1.0,
    1.0, 0.0
];


impl FromWorld for PointCloudPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let instanced_point_quad = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: "instanced point quad".into(),
            contents: unsafe {
                std::slice::from_raw_parts(QUAD_VERTEX_BUF.as_ptr() as *const _, std::mem::size_of_val(QUAD_VERTEX_BUF))
            },
            usage: BufferUsages::VERTEX,
        });
        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: "Eye Dome Shadingd Sampler".into(),
            ..Default::default()
        });
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
        let entity_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("PointCloudViewLabel"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None
                }
            ],
        });
        let eye_dome_image_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("EyeDomeImageLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: TextureSampleType::Depth, view_dimension: TextureViewDimension::D2, multisampled: true },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture { access: StorageTextureAccess::ReadWrite, format: TextureFormat::Rgba8Unorm, view_dimension: TextureViewDimension::D2 },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None
                },
            ],
        });
        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("point_cloud_pipeline".into()),
            layout: Some(vec![
                view_layout.clone(),
                entity_layout.clone(),
            ]),
            vertex: VertexState {
                shader: POINT_CLOUD_VERT_SHADER_HANDLE.typed(),
                shader_defs: Default::default(),
                entry_point: "main".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }
                ],
            },
            fragment: Some(FragmentState {
                shader: POINT_CLOUD_FRAG_SHADER_HANDLE.typed(),
                shader_defs: Default::default(),
                entry_point: "main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleStrip,
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

        let eye_dome_compute_pipeline_descriptor = ComputePipelineDescriptor {
            label: Some("EyeDomeLightingPipeline".into()),
            layout: Some(vec![
                eye_dome_image_layout.clone()
            ]),
            shader: EYE_DOME_LIGHTING_SHADER_HANDLE.typed(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
        };

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(pipeline_descriptor);
        let eye_dome_pipeline_id = pipeline_cache.queue_compute_pipeline(eye_dome_compute_pipeline_descriptor);

        Self {
            pipeline_id,
            view_layout,
            entity_layout,
            eye_dome_pipeline_id,
            eye_dome_image_layout,
            sampler,
            instanced_point_quad
        }
    }
}


#[derive(Clone, Component)]
struct EyeDomeViewTarget {
    depth_texture: Texture,
    depth_texture_view: TextureView,
    bind_group: BindGroup,
}

fn prepare_view_targets(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    pipeline: Res<PointCloudPipeline>,
    cameras: Query<(Entity, &ExtractedCamera, &ExtractedView, &ViewTarget)>,
) {
    let mut textures = HashMap::default();
    for (entity, camera, view, view_target) in cameras.iter() {
        if let Some(target_size) = camera.physical_target_size {
            let size = Extent3d {
                width: target_size.x,
                height: target_size.y,
                depth_or_array_layers: 1,
            };

            let main_textures = textures
                .entry(camera.target.clone())
                .or_insert_with(|| {
                    let depth_descriptor = TextureDescriptor {
                        label: None,
                        size,
                        mip_level_count: 1,
                        sample_count: 4,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Depth32Float,
                        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    };
                    let cached_depth_texture = texture_cache.get(&render_device, depth_descriptor);


                    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                        label: "Eye Dome Bind Group".into(),
                        layout: &pipeline.eye_dome_image_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: bevy::render::render_resource::BindingResource::TextureView(&cached_depth_texture.default_view),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: bevy::render::render_resource::BindingResource::TextureView(&view_target.main_texture()),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: bevy::render::render_resource::BindingResource::Sampler(&pipeline.sampler),
                            },
                        ],
                    });
                    EyeDomeViewTarget {
                        depth_texture: cached_depth_texture.texture,
                        depth_texture_view: cached_depth_texture.default_view,
                        bind_group
                    }
                });

            commands.entity(entity).insert(main_textures.clone());
        }
    }
}
