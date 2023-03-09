use std::future::Future;

use crate::PointCloudAsset;
use bevy::asset::LoadedAsset;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{asset::AssetLoader, prelude::Mesh};

pub struct OpdLoader;

impl OpdLoader {
    pub fn load_opd<'a>(bytes: &'a [u8]) -> impl Future<Output = PointCloudAsset> + 'a {
        async move {
            let file = opd_parser::parse(bytes).unwrap().1;
            let mut positions = Vec::new();

            let mut max_position = [f32::MIN, f32::MIN, f32::MIN];
            let mut min_position = [f32::MAX, f32::MAX, f32::MAX];

            for i in file.centroids.into_iter() {
                let pos: [f32; 3] = i.offset.into();
                for i in 0..3 {
                    max_position[i] = max_position[i].max(pos[i]);
                    min_position[i] = min_position[i].min(pos[i]);
                }
                positions.push(pos);
            }

            let size = [
                max_position[0] - min_position[0],
                max_position[1] - min_position[1],
                max_position[2] - min_position[2],
            ];
            for position in positions.iter_mut() {
                for i in 0..3 {
                    position[i] = position[i] - min_position[i] - size[i] / 2.0;
                }
            }

            let mut mesh = Mesh::new(PrimitiveTopology::PointList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

            let animation_scale = file.header.directive.scale;
            let asset = PointCloudAsset {
                mesh,
                animation: Some(file.frames),
                animation_scale,
            };
            asset
        }
    }
}

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
            let asset = Self::load_opd(bytes).await;
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }
}
