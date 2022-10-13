use crate::Cuboid;
use bevy::render::render_resource::UniformBuffer;
use bevy::{
    prelude::*,
    render::render_resource::{BindGroup},
    utils::HashMap,
};

#[derive(Default)]
pub(crate) struct CuboidBufferCache {
    pub entries: HashMap<Entity, CachedCuboidBuffers>,
}

#[derive(Default)]
pub(crate) struct CachedCuboidBuffers {
    pub color_options_index: u32,
    pub dirty: bool,
    pub enabled: bool,
    pub keep_alive: bool,
    pub instance_buffer: UniformBuffer<[Cuboid;1]>,
    pub instance_buffer_bind_group: Option<BindGroup>,
    pub position: Vec3,
    pub transform_index: u32,
}

impl CuboidBufferCache {
    pub fn cull_entities(&mut self) {
        let mut to_remove = Vec::new();
        for (entity, entry) in self.entries.iter_mut() {
            if !entry.keep_alive {
                to_remove.push(*entity);
            }
            entry.keep_alive = false;
        }
        for entity in to_remove {
            self.entries.remove(&entity);
        }
    }
}
