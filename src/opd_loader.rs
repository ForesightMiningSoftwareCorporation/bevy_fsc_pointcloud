use crate::PointCloudAsset;
use bevy::asset::LoadedAsset;
use bevy::math::Vec3A;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{asset::AssetLoader, prelude::Mesh};

pub struct OpdLoader;

impl OpdLoader {
    pub async fn load_opd<'a>(bytes: &'a [u8]) -> Result<PointCloudAsset, anyhow::Error> {
        let file = opd_parser::parse(bytes).map_err(|e| e.to_owned())?.1;
        let mut positions = Vec::new();

        let mut max_position = Vec3A::new(f32::MIN, f32::MIN, f32::MIN);
        let mut min_position = Vec3A::new(f32::MAX, f32::MAX, f32::MAX);

        for i in file.centroids.into_iter() {
            let pos: [f32; 3] = i.offset.into();

            max_position = max_position.max(i.offset.into());
            min_position = min_position.min(i.offset.into());
            positions.push(pos);
        }

        let size = max_position - min_position;
        let position_offset: [f32; 3] = (min_position + size / 2.0).into();
        for position in positions.iter_mut() {
            for i in 0..3 {
                position[i] = position[i] - position_offset[i];
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::PointList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        Ok(PointCloudAsset {
            mesh,
            animation: Some(file.frames),
            animation_scale: file.header.directive.scale,
        })
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
            let asset = Self::load_opd(bytes).await?;
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }
}
