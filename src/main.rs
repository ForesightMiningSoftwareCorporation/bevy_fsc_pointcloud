use bevy::prelude::*;
use bevy_potree::{PointCloudAsset, PotreePointCloud, PointCloudPipelineConfig};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin::default()))
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .insert_resource(Msaa { samples: 1 })
        .add_plugin(bevy_potree::PointCloudPlugin {
            colored: false,
            animated: true
        })
        .add_startup_system(startup);
    app.insert_resource(PointCloudPipelineConfig {
        colored: false,
    });
    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mesh: Handle<PointCloudAsset> = asset_server.load("replay5.opd");

    commands
        .spawn(PotreePointCloud {
            mesh,
            point_size: 2.0,
        })
        .insert(Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)))
        .insert(GlobalTransform::default());

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(1.0, 1.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(FpsCameraBundle::new(
            Default::default(),
            Vec3::new(30.0, 30.0, 30.0),
            Vec3::ZERO,
        ));
}
