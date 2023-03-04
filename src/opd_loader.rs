use std::path::{PathBuf, Path};

use bevy::{asset::AssetLoader, prelude::Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::asset::LoadedAsset;
use crate::PointCloudAsset;
use bevy::math::Vec3;
pub struct OpdLoader;

impl AssetLoader for OpdLoader {
    fn extensions(&self) -> &[&str] {
        &["opd"]
    }
    fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
            Box::pin(async move {
                let mut file = opd_parser::parse(bytes).unwrap().1;
                let mut positions = Vec::new();

                let mut max_position = [f32::MIN, f32::MIN, f32::MIN];
                let mut min_position = [f32::MAX, f32::MAX, f32::MAX];

                for i in file.centroids.into_iter() {
                    let mut pos: [f32; 3] = i.offset.into();
                    for i in 0..3 {
                        max_position[i] = max_position[i].max(pos[i]);
                        min_position[i] = min_position[i].min(pos[i]);
                    }
                    positions.push(pos);
                }

                println!("{:?}", file.header.directive.scale);
                let size = [max_position[0] - min_position[0], max_position[1] - min_position[1], max_position[2] - min_position[2]];

                for position in positions.iter_mut() {
                    for i in 0..3 {
                        position[i] = (position[i] - min_position[i]) / size[i];
                    }
                    position.swap(1, 2);
                }
                let mut mesh = Mesh::new(PrimitiveTopology::PointList);
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                

                std::mem::swap(&mut  file.header.directive.scale.y, &mut  file.header.directive.scale.z);
                let animation_scale =  file.header.directive.scale / Vec3::from(size);
                let asset = PointCloudAsset {
                    mesh,
                    animation: Some(file.frames),
                    animation_scale
                };
                load_context.set_default_asset(LoadedAsset::new(asset));
                Ok(())
            })
    }
}
