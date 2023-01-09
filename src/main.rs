use bevy::{prelude::*, window::PresentMode};
use bevy_potree::{PointCloudAsset, PotreePointCloud};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            present_mode: PresentMode::Immediate,
            ..Default::default()
        },
        ..Default::default()
    }))
    .add_plugin(bevy_potree::PointCloudPlugin::default())
    .add_startup_system(startup);
    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Starting");
    let _path = std::env::args().skip(1).next();
    let mesh: Handle<PointCloudAsset> = asset_server.load("points.las");

    commands
        .spawn(SpatialBundle::default())
        .insert(PotreePointCloud { mesh });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(1.0, 1.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
