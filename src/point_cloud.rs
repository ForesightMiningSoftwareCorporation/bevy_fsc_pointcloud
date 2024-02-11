use bevy::prelude::*;
use opd_parser::Frames;

#[derive(Clone, Asset, TypePath)]
pub struct PointCloudAsset {
    pub mesh: Mesh,
    pub animation: Option<Frames>,
    pub animation_scale: Vec3,
}

impl PointCloudAsset {
    pub fn animation_duration(&self) -> Option<f32> {
        match &self.animation {
            Some(Frames::I8(frames)) => Some(frames.last().unwrap().time / 1000.),
            _ => None,
        }
    }
}
