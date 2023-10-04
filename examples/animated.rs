use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_fsc_point_cloud::{PointCloudAsset, PointCloudPlaybackControls, PotreePointCloud};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin::default()),
            EguiPlugin,
            bevy_fsc_point_cloud::PointCloudPlugin,
            FpsCameraPlugin::default(),
            LookTransformPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, controls_window)
        .run();
}

#[derive(Resource)]
struct PointCloud(Handle<PointCloudAsset>);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle::default(),
        FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 200.0,
                ..Default::default()
            },
            Vec3::new(0.0, 100.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            Vec3::Y,
        ),
    ));

    let point_cloud: Handle<PointCloudAsset> = asset_server.load("replay.opd");

    commands
        .spawn(PotreePointCloud {
            mesh: point_cloud.clone(),
            point_size: 1.0,
        })
        .insert(SpatialBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(
                -std::f32::consts::FRAC_PI_2,
            )),
            ..Default::default()
        });

    commands.insert_resource(PointCloud(point_cloud));
}

fn controls_window(
    mut ctx: EguiContexts,
    pc: Res<PointCloud>,
    mut controls: ResMut<PointCloudPlaybackControls>,
    assets: Res<Assets<PointCloudAsset>>,
) {
    let controls = controls.controls_mut(&pc.0);
    let Some(asset) = assets.get(&pc.0) else {
        return;
    };
    egui::Window::new("Animation Controls").show(ctx.ctx_mut(), |ui| {
        if controls.playing {
            if ui.button("Stop").clicked() {
                controls.playing = false;
            }
        } else if ui.button("Start").clicked() {
            controls.playing = true;
        }

        let animation_duration = asset.animation_duration().unwrap();

        ui.add(egui::Slider::new(&mut controls.speed, -1.0..=10.0).text("Speed"));
        ui.add(egui::Slider::new(&mut controls.time, 0.0..=animation_duration).text("Seek"));
    });
}
