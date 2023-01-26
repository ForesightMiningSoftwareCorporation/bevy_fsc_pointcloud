use bevy::{
    prelude::*,
    window::PresentMode,
};
use bevy_potree::{PointCloudAsset, PotreePointCloud};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraPlugin}, LookTransformPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            present_mode: PresentMode::Immediate,
            ..Default::default()
        }),
        ..Default::default()
    }))
    .add_plugin(LookTransformPlugin)
    .add_plugin(OrbitCameraPlugin::default())
    .add_plugin(bevy_potree::PointCloudPlugin::default())
    .add_startup_system(startup);
    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Starting");
    let _path = std::env::args().skip(1).next();
    let mesh: Handle<PointCloudAsset> = asset_server.load("points.laz");

    commands
        .spawn(SpatialBundle::default())
        .insert(PotreePointCloud { mesh });

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(1.0, 1.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCameraBundle::new(
            Default::default(),
            Vec3::new(3.0, 3.0, 3.0),
            Vec3::ZERO,
        ));
}
