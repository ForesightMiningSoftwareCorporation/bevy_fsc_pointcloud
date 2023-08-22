use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use bevy_fsc_point_cloud::{PointCloudAsset, PointCloudPlaybackControl, PotreePointCloud};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin::default()))
        .add_plugin(PlayerPlugin)
        .add_plugin(bevy_fsc_point_cloud::PointCloudPlugin)
        .insert_resource(PointCloudPlaybackControl {
            speed: 1.0,
            ..Default::default()
        })
        .add_startup_system(startup);
    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mesh: Handle<PointCloudAsset> = asset_server.load("replay.opd");

    commands
        .spawn(PotreePointCloud {
            mesh,
            point_size: 3.0,
        })
        .insert(SpatialBundle::default());
}
