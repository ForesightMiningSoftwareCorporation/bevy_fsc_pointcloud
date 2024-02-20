use bevy::{prelude::*, utils::HashMap};

use crate::PointCloudAsset;

#[derive(Resource, Clone, Default)]
pub struct PointCloudPlaybackControls {
    pub(crate) controls: HashMap<Handle<PointCloudAsset>, PlaybackControls>,
}

#[derive(Clone, Copy)]
pub struct PlaybackControls {
    pub time: f32,
    pub playing: bool,
    pub speed: f32,
}

impl Default for PlaybackControls {
    fn default() -> Self {
        Self {
            time: 0.,
            playing: false,
            speed: 1.,
        }
    }
}

impl PointCloudPlaybackControls {
    pub fn controls(&self, handle: &Handle<PointCloudAsset>) -> PlaybackControls {
        self.controls.get(handle).copied().unwrap_or_default()
    }

    pub fn controls_mut(&mut self, handle: &Handle<PointCloudAsset>) -> &mut PlaybackControls {
        self.controls.entry(handle.clone_weak()).or_default()
    }

    pub fn playback_system(
        mut controls: ResMut<Self>,
        time: Res<Time>,
        assets: Res<Assets<PointCloudAsset>>,
    ) {
        let mut changed = false;
        let len = controls.controls.len();

        controls
            .bypass_change_detection()
            .controls
            .retain(|handle, controls| {
                let Some(animation_duration) = assets
                    .get(handle)
                    .and_then(|asset| asset.animation_duration())
                else {
                    // remove if asset doesn't exist or isn't animated
                    return false;
                };

                if controls.playing {
                    changed |= true;
                    controls.time += controls.speed * time.delta_seconds();
                    controls.time = controls.time.rem_euclid(animation_duration);
                }

                true
            });

        if changed || controls.controls.len() != len {
            controls.set_changed();
        }
    }
}
