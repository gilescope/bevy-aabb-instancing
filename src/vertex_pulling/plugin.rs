use super::cuboid_cache::CuboidBufferCache;
use super::draw::{AuxiliaryMeta, DrawCuboids, TransformsMeta, ViewMeta};
use super::extract::{extract_clipping_planes, extract_cuboids};
use super::index_buffer::CuboidsIndexBuffer;
use super::pipeline::{CuboidsPipeline, CuboidsShaderDefs, VERTEX_PULLING_SHADER_HANDLE};
use super::prepare::{
    prepare_auxiliary_bind_group, prepare_clipping_planes, prepare_color_options,
    prepare_cuboid_transforms, prepare_cuboids, prepare_cuboids_index_buffer,
    prepare_cuboids_view_bind_group,
};
use super::queue::queue_cuboids;
use crate::clipping_planes::GpuClippingPlaneRanges;
use crate::cuboids::CuboidsTransform;
use crate::{ColorOptions, ColorOptionsMap};

use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::{
    render_phase::AddRenderCommand,
    render_resource::{DynamicUniformBuffer, UniformBuffer},
    RenderApp, RenderStage,
};

/// Renders the [`Cuboids`](crate::Cuboids) component using the "vertex pulling" technique.
#[derive(Default)]
pub struct VertexPullingRenderPlugin {
    pub outlines: bool,
}

impl Plugin for VertexPullingRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorOptionsMap>();

        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            VERTEX_PULLING_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("vertex_pulling.wgsl")),
        );

        let maybe_msaa = app.world.get_resource::<Msaa>().cloned();
        let render_app = app.sub_app_mut(RenderApp);

        if let Some(msaa) = maybe_msaa {
            render_app.insert_resource(msaa);
        }
        let mut shader_defs = CuboidsShaderDefs::default();
        // if self.outlines {
        //     shader_defs.enable_outlines();
        // }
        render_app.insert_resource(shader_defs);

        render_app
            .add_render_command::<Opaque3d, DrawCuboids>()
            .init_resource::<AuxiliaryMeta>()
            .init_resource::<CuboidBufferCache>()
            .init_resource::<CuboidsIndexBuffer>()
            .init_resource::<CuboidsPipeline>()
            .init_resource::<DynamicUniformBuffer<ColorOptions>>()
            .init_resource::<DynamicUniformBuffer<CuboidsTransform>>()
            .init_resource::<TransformsMeta>()
            .init_resource::<UniformBuffer<GpuClippingPlaneRanges>>()
            .init_resource::<ViewMeta>()
            .add_system_to_stage(RenderStage::Extract, extract_cuboids)
            .add_system_to_stage(RenderStage::Extract, extract_clipping_planes)
            .add_system_to_stage(RenderStage::Prepare, prepare_color_options)
            .add_system_to_stage(RenderStage::Prepare, prepare_clipping_planes)
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_auxiliary_bind_group
                    .after(prepare_color_options)
                    .after(prepare_clipping_planes),
            )
            .add_system_to_stage(RenderStage::Prepare, prepare_cuboids_index_buffer)
            .add_system_to_stage(RenderStage::Prepare, prepare_cuboid_transforms)
            .add_system_to_stage(RenderStage::Prepare, prepare_cuboids)
            // HACK: prepare view bind group should happen in prepare phase, but
            // ViewUniforms resource is not ready until after prepare phase;
            // need system order/label exported from bevy
            .add_system_to_stage(RenderStage::Queue, prepare_cuboids_view_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_cuboids);
    }
}
