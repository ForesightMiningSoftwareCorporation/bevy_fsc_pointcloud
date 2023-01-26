use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::ExtractedCamera,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferUsages, CachedComputePipelineId,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction,
            ComputePipelineDescriptor, DepthBiasState, DepthStencilState, Extent3d, FragmentState,
            FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, StencilFaceState, StencilState, StorageTextureAccess,
            Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
            TextureUsages, TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::TextureCache,
        view::{ExtractedView, ViewTarget, ViewUniforms},
    },
    utils::HashMap,
};

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
    pub bind_group: Option<BindGroup>,
}
pub(crate) fn prepare_point_cloud_bind_group(
    render_device: Res<RenderDevice>,
    pipeline: Res<PointCloudPipeline>,
    view_uniform: Res<ViewUniforms>,
    mut bind_groups: ResMut<PointCloudBindGroup>,
) {
    if let Some(resource) = view_uniform.uniforms.binding() {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("point_cloud_bind_group"),
            layout: &pipeline.view_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource,
            }],
        });
        bind_groups.bind_group = Some(bind_group);
    }
}

const QUAD_VERTEX_BUF: &'static [f32] = &[0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];

impl FromWorld for PointCloudPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let instanced_point_quad = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: "instanced point quad".into(),
            contents: unsafe {
                std::slice::from_raw_parts(
                    QUAD_VERTEX_BUF.as_ptr() as *const _,
                    std::mem::size_of_val(QUAD_VERTEX_BUF),
                )
            },
            usage: BufferUsages::VERTEX,
        });
        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: "Eye Dome Shadingd Sampler".into(),
            ..Default::default()
        });
        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("PointCloudViewLabel"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let entity_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("PointCloudViewLabel"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let eye_dome_image_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("EyeDomeImageLayout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Depth,
                            view_dimension: TextureViewDimension::D2,
                            multisampled: true,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("point_cloud_pipeline".into()),
            layout: Some(vec![view_layout.clone(), entity_layout.clone()]),
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
                }],
            },
            fragment: Some(FragmentState {
                shader: POINT_CLOUD_FRAG_SHADER_HANDLE.typed(),
                shader_defs: Default::default(),
                entry_point: "main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
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
            layout: Some(vec![eye_dome_image_layout.clone()]),
            shader: EYE_DOME_LIGHTING_SHADER_HANDLE.typed(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
        };

        let pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(pipeline_descriptor);
        let eye_dome_pipeline_id =
            pipeline_cache.queue_compute_pipeline(eye_dome_compute_pipeline_descriptor);

        Self {
            pipeline_id,
            view_layout,
            entity_layout,
            eye_dome_pipeline_id,
            eye_dome_image_layout,
            sampler,
            instanced_point_quad,
        }
    }
}

#[derive(Clone, Component)]
pub struct EyeDomeViewTarget {
    pub depth_texture: Texture,
    pub depth_texture_view: TextureView,
    pub bind_group: BindGroup,
}

pub(crate) fn prepare_view_targets(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    pipeline: Res<PointCloudPipeline>,
    cameras: Query<(Entity, &ExtractedCamera, &ExtractedView, &ViewTarget)>,
) {
    let mut textures = HashMap::default();
    for (entity, camera, _view, view_target) in cameras.iter() {
        if let Some(target_size) = camera.physical_target_size {
            let size = Extent3d {
                width: target_size.x,
                height: target_size.y,
                depth_or_array_layers: 1,
            };

            let main_textures = textures.entry(camera.target.clone()).or_insert_with(|| {
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
                            resource: bevy::render::render_resource::BindingResource::TextureView(
                                &cached_depth_texture.default_view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: bevy::render::render_resource::BindingResource::TextureView(
                                &view_target.main_texture(),
                            ),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: bevy::render::render_resource::BindingResource::Sampler(
                                &pipeline.sampler,
                            ),
                        },
                    ],
                });
                EyeDomeViewTarget {
                    depth_texture: cached_depth_texture.texture,
                    depth_texture_view: cached_depth_texture.default_view,
                    bind_group,
                }
            });

            commands.entity(entity).insert(main_textures.clone());
        }
    }
}
