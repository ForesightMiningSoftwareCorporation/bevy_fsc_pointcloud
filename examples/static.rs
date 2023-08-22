use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use bevy_fsc_point_cloud::{
    ClippingPlaneBundle, ClippingPlaneRange, PointCloudAsset, PotreePointCloud,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin::default()))
        .add_plugin(PlayerPlugin)
        .add_plugin(bevy_fsc_point_cloud::PointCloudPlugin)
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
