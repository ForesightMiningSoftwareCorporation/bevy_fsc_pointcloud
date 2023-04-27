use bevy::prelude::*;
use bevy_fsc_point_cloud::{
    ClippingPlaneBundle, ClippingPlaneRange, PointCloudAsset, PotreePointCloud,
};
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
        .add_plugin(bevy_fsc_point_cloud::PointCloudPlugin {
            colored: true,
            animated: false,
        })
        .add_startup_system(startup);
    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mesh: Handle<PointCloudAsset> = asset_server.load("laman_mahkota.laz");

    commands
        .spawn(PotreePointCloud {
            mesh,
            point_size: 0.007,
        })
        .insert(SpatialBundle::default());

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(1.0, 1.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(FpsCameraBundle::new(
            Default::default(),
            Vec3::new(3.0, 3.0, 3.0),
            Vec3::ZERO,
        ));

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
