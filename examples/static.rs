use bevy::prelude::*;
use bevy_fsc_point_cloud::{
    ClippingPlaneBundle, ClippingPlaneRange, PointCloudAsset, PotreePointCloud,
};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin::default()),
            LookTransformPlugin,
            FpsCameraPlugin::default(),
            bevy_fsc_point_cloud::PointCloudPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 200.0,
                ..Default::default()
            },
            Vec3::new(0.0, 100.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            Vec3::Y,
        ));

    let mesh: Handle<PointCloudAsset> = asset_server.load("laman_mahkota.laz");

    commands
        .spawn(PotreePointCloud {
            mesh,
            point_size: 0.007,
        })
        .insert(SpatialBundle::default());

    commands.spawn(ClippingPlaneBundle {
        range: ClippingPlaneRange {
            min_sdist: 0.0,
            max_sdist: 0.5,
        },
        transform: TransformBundle {
            local: Transform::from_translation(Vec3 {
                x: 0.0,
                y: 30.0,
                z: 0.0,
            }),
            global: Default::default(),
        },
    });
}
