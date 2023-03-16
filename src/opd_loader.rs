use crate::PointCloudAsset;
use bevy::asset::LoadedAsset;
use bevy::math::Vec3A;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{asset::AssetLoader, prelude::Mesh};

pub struct OpdLoader;

impl OpdLoader {
    pub async fn load_opd(bytes: &[u8]) -> Result<PointCloudAsset, anyhow::Error> {
        let file = opd_parser::parse(bytes).map_err(|e| e.to_owned())?.1;
        let mut positions: Vec<Vec3A> = Vec::new();

        let mut max_position = Vec3A::splat(f32::MIN);
        let mut min_position = Vec3A::splat(f32::MAX);

        for i in file.centroids {
            max_position = max_position.max(i.offset.into());
            min_position = min_position.min(i.offset.into());
            positions.push(i.offset.into());
        }

        let size = max_position - min_position;
        let position_offset: Vec3A = min_position + size / 2.0;
        for position in positions.iter_mut() {
            *position -= position_offset;
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
