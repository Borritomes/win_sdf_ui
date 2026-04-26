use bevy::{core_pipeline::{Core2d, Core2dSystems}, prelude::*, render::{RenderApp, extract_component::{ExtractComponent, ExtractComponentPlugin}, render_resource::ShaderType, renderer::ViewQuery, view::{ExtractedWindows, ViewTarget}}};

pub struct RenderToWindowPlugin;

impl Plugin for RenderToWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<RenderToWindow>::default());

        let Some(mut render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("failed to get RenderApp");
            return
        };

        render_app.add_systems(
            Core2d,
            render_to_window_system.in_set(Core2dSystems::PostProcess)
        );
    }
}

#[derive(Component, Default, Reflect, Clone, Copy, ExtractComponent)]
#[reflect(Component)]
pub struct RenderToWindow;

pub fn render_to_window_system(
    view: ViewQuery<(
        &ViewTarget,
        &RenderToWindow,
    )>,
    windows: If<Res<ExtractedWindows>>
) {
    let Some((_, window)) = windows.iter().next() else {
        warn!("failed to get window");
        return
    };

    let (view_target, render_to_window) = view.into_inner();

    
}