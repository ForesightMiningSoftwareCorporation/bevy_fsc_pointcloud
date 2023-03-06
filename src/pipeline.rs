use std::num::NonZeroU64;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::ExtractedCamera,
        extract_component::ComponentUniforms,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent,
            BlendFactor, BlendOperation, BlendState, Buffer, BufferBindingType,
            BufferInitDescriptor, BufferUsages, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Extent3d,
            FragmentState, FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, StencilFaceState, StencilState, Texture,
            TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
            TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode, CachedComputePipelineId, ComputePipelineDescriptor,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, TextureCache},
        view::{ExtractedView, ViewTarget, ViewUniforms},
        RenderApp, render_asset::RenderAssets, extract_resource::ExtractResource,
    },
    utils::HashMap, animation,
};

use crate::{PointCloudUniform, pipeline, PointCloudAsset};

pub(crate) const POINT_CLOUD_VERT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf01);
pub(crate) const POINT_CLOUD_FRAG_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf02);
pub(crate) const EYE_DOME_LIGHTING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf03);
pub(crate) const ANIMATION_COMPUTE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3fc9d1ff70cedf04);

#[derive(Resource)]
pub struct PointCloudPipeline {
    pub pipeline_id: CachedRenderPipelineId,
    pub view_layout: BindGroupLayout,
    pub entity_layout: BindGroupLayout,
    pub model_layout: BindGroupLayout,

    pub eye_dome_pipeline_id: CachedRenderPipelineId,
    pub eye_dome_image_layout: BindGroupLayout,

    pub animation_compute_layout: BindGroupLayout,
    pub animation_compute_pipeline_id: Option<CachedComputePipelineId>,

    pub sampler: Sampler,
    pub instanced_point_quad: Buffer,

    pub colored: bool,
    pub animated: bool,
}

#[derive(Resource, Default)]
pub struct PointCloudBindGroup {
    pub bind_group: Option<BindGroup>,
    pub model_bind_group: Option<BindGroup>,
}
pub(crate) fn prepare_point_cloud_bind_group(
    render_device: Res<RenderDevice>,
    pipeline: Res<PointCloudPipeline>,
    view_uniform: Res<ViewUniforms>,
    model_uniform: Res<ComponentUniforms<PointCloudUniform>>,
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

    if let Some(binding) = model_uniform.uniforms().binding() {
        bind_groups.model_bind_group =
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("point_cloud_model_bind_group"),
                layout: &pipeline.model_layout,
            }));
    }
}

const QUAD_VERTEX_BUF: &'static [f32] = &[0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];

#[derive(Clone, Resource)]
pub struct PointCloudPipelineConfig {
    pub colored: bool
}
impl Default for PointCloudPipelineConfig {
    fn default() -> Self {
        Self {
            colored: true,
        }
    }
}
impl PointCloudPipeline {
    pub fn from_app(app: &mut App, colored: bool, animated: bool) -> Self {
        let msaa = app
            .world
            .get_resource::<Msaa>()
            .map(|a| a.samples)
            .unwrap_or(1);
        let render_app = app.sub_app_mut(RenderApp);
        let world = &mut render_app.world;
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
            label: "Eye Dome Shading Sampler".into(),
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
            label: Some("PointCloudViewLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }].as_slice()[if animated {
                0..2
            } else {
                0..1
            }],
        });
        let model_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("PointCloudModelLayout"),
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
        let eye_dome_image_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("EyeDomeImageLayout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Depth,
                            view_dimension: TextureViewDimension::D2,
                            multisampled: msaa > 1,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
            
        let animation_compute_layout =
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("AnimationComputeLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("point_cloud_pipeline".into()),
            layout: Some(vec![
                view_layout.clone(),
                entity_layout.clone(),
                model_layout.clone(),
            ]),
            vertex: VertexState {
                shader: POINT_CLOUD_VERT_SHADER_HANDLE.typed(),
                shader_defs: {
                    let mut defs = Vec::new();
                    if colored { defs.push("COLORED".to_string()) };
                    if animated { defs.push("ANIMATED".to_string()) };
                    defs
                },
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
                shader_defs: {
                    let mut defs = Vec::new();
                    if colored { defs.push("COLORED".to_string()) };
                    if animated { defs.push("ANIMATED".to_string()) };
                    defs
                },
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
                count: msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        let eye_dome_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("EyeDomeLightingPipeline".into()),
            layout: Some(vec![eye_dome_image_layout.clone()]),
            vertex: VertexState {
                shader: EYE_DOME_LIGHTING_SHADER_HANDLE.typed(),
                shader_defs: if msaa > 1 {
                    vec!["MULTISAMPLED".to_string()]
                } else {
                    Vec::new()
                },
                entry_point: "vertex".into(),
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
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: EYE_DOME_LIGHTING_SHADER_HANDLE.typed(),
                shader_defs: if msaa > 1 {
                    vec!["MULTISAMPLED".to_string()]
                } else {
                    Vec::new()
                },
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::Zero,
                            dst_factor: BlendFactor::SrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::Zero,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::COLOR,
                })],
            }),
        };

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(pipeline_descriptor);
        let eye_dome_pipeline_id =
            pipeline_cache.queue_render_pipeline(eye_dome_pipeline_descriptor);

        let animation_compute_pipeline_id = if animated {
            Some(pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec! [
                animation_compute_layout.clone()
            ]),
            shader: ANIMATION_COMPUTE_SHADER_HANDLE.typed(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
        })) } else {
            None
        };

        Self {
            pipeline_id,
            view_layout,
            model_layout,
            entity_layout,
            eye_dome_pipeline_id,
            eye_dome_image_layout,
            sampler,
            instanced_point_quad,
            colored,
            animated,
            animation_compute_layout,
            animation_compute_pipeline_id
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
    msaa: Option<Res<Msaa>>,
) {
    let msaa = msaa.map(|a| a.samples).unwrap_or(1);
    let mut textures = HashMap::default();
    for (entity, camera, _view, _view_target) in cameras.iter() {
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
                    sample_count: msaa,
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

#[derive(Resource, Clone)]
pub struct PointCloudPlaybackControl {
    pub playing: bool,
    progress: f32,
}
impl PointCloudPlaybackControl {
    pub fn progress(&self) -> f32 {
        self.progress
    }
}
impl Default for PointCloudPlaybackControl {
    fn default() -> Self {
        Self {
            playing: true,
            progress: 0.0,
        }
    }
}

impl ExtractResource for PointCloudPlaybackControl {
    type Source = Self;

    /// Defines how the resource is transferred into the "render world".
    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

pub fn prepare_animated_assets(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut assets: ResMut<RenderAssets<PointCloudAsset>>,
    mut playback: ResMut<PointCloudPlaybackControl>,
    time: Res<Time>
) {
    if !playback.playing {
        return;
    }
    for (handle, asset) in assets.iter_mut() {
        if let Some(staging) = asset.animation_buffer_staging.as_mut() {
            let size = asset.num_points as usize * std::mem::size_of::<f32>() * 3;
            let mut view = queue.write_buffer_with(asset.animation_buffer_staging.as_ref().unwrap(), 0, NonZeroU64::new(size as u64).unwrap());
            let view = &mut *view;
            let view = unsafe { std::slice::from_raw_parts_mut(view.as_mut_ptr() as *mut f32, asset.num_points as usize * std::mem::size_of::<f32>() * 3)};
            
            let frame_count = match asset.frames.as_ref().unwrap() {
                opd_parser::Frames::U8(frames) => {
                    let duration = frames.last().as_ref().unwrap().time / 1000.0;
                    let current_frame = &frames[asset.current_animation_frame];
                    if current_frame.time / 1000.0 > time.elapsed_seconds_wrapped() - asset.animation_start_time {
                        asset.requires_update = false;
                        continue;
                    }
                    asset.requires_update = true;
                    playback.progress = (time.elapsed_seconds_wrapped() - asset.animation_start_time) / duration;

                    asset.current_animation_frame += 1;
                    if asset.current_animation_frame >= frames.len() {
                        asset.current_animation_frame = 0;
                        asset.animation_start_time = time.elapsed_seconds_wrapped();
                    }

                    let scale: [f32; 3] = asset.animation_scale.into();
                    let mut iter = frames[asset.current_animation_frame].data.iter().enumerate();
                    loop {
                        let mut arr = [
                            match iter.next() {
                                Some(a) => a,
                                None => break
                            },
                            match iter.next() {
                                Some(a) => a,
                                None => break
                            },
                            match iter.next() {
                                Some(a) => a,
                                None => break
                            }
                        ];
                        arr.swap(1, 2);
                        for j in 0..3 {
                            let (i, value) = arr[j];
                            let value = *value;
                            let value: i8 = unsafe { std::mem::transmute(value)};
                            let value: f32 = value as f32 / i8::MAX as f32;
                            let value = value * 0.1;

                            let scale = scale[j];
                            view[i] = value * scale;
                        }
                    };

                    frames.len()
                },
                opd_parser::Frames::U16(_) => todo!(),
                opd_parser::Frames::U32(_) => todo!(),
                opd_parser::Frames::U64(_) => todo!(),
            };

            
        }
    }
}
