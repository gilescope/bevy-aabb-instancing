use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType},
};

use crate::ColorOptionsId;

/// Value that determines the color of a [`Cuboid`] based on the associated
/// [`ColorOptions`](crate::ColorOptions).
pub type Color = u32;

/// Metadata encoded in 32 bits:
///
/// - `0x000000FF` = 0 for visible or 1 for invisible
/// - `0x0000FF00` = unused
/// - `0xFFFF0000` = depth bias (u16)
///   - Multiplies the depth of each cuboid vertex by `1 + bias * eps` where
///     `eps = 8e-8`. This can be used with random biases to avoid Z-fighting.
pub type MetaBits = u32;

/// An axis-aligned box, extending from `minimum` to `maximum`.
#[derive(Clone, Copy, Debug, ShaderType, Default)]
#[repr(C)]
pub struct Cuboid {
    pub minimum_x: f32,
    pub minimum_y: f32,
    pub minimum_z: f32,
    pub meta_bits: MetaBits,
    pub maximum_x: f32,
    pub maximum_y: f32,
    pub maximum_z: f32,
    pub color: Color,
}

impl Cuboid {
    pub fn new(minimum: Vec3, maximum: Vec3, color: u32, visible: bool, depth_bias: u16) -> Self {
        assert_eq!(std::mem::size_of::<Cuboid>(), 32);
        assert_eq!(std::mem::size_of::<[Cuboid;1]>(), 32);
        let mut me = Self {
            minimum_x: minimum.x,
            minimum_y: minimum.y,
            minimum_z: minimum.z,
            meta_bits: 0,
            maximum_x: maximum.x,
            maximum_y: maximum.y,
            maximum_z: maximum.z,
            color: color.clone(),
        };
        if visible {
            me.make_visible();
        } else {
            me.make_invisible();
        }
        me.set_depth_bias(depth_bias);
        me
    }

    #[inline]
    pub fn make_visible(&mut self) {
        self.meta_bits &= !1;
    }

    #[inline]
    pub fn make_invisible(&mut self) {
        self.meta_bits |= 1;
    }

    #[inline]
    pub fn set_depth_bias(&mut self, bias: u16) {
        self.meta_bits &= !0xFFFF0000; // clear
        self.meta_bits |= (bias as u32) << 16; // set
    }
}

/// A set of cuboids to be extracted for rendering.
#[derive(Clone, Component, Debug)]
pub struct Cuboids {
    /// Instances to be rendered.
    pub instances: [Cuboid;1],
}

impl Cuboids {
    pub fn new(instances: Vec<Cuboid>) -> Self {
        Self { instances: [instances[0]] }//TODO
    }

    /// Automatically creates an [`Aabb`] that bounds all `instances`.
    pub fn aabb(&self) -> Aabb {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        for i in self.instances.iter() {
            min_x = min_x.min(i.minimum_x);
            max_x = max_x.max(i.maximum_x);
            min_y = min_y.min(i.minimum_y);
            max_y = max_y.max(i.maximum_y);
            min_z = min_z.min(i.minimum_z);
            max_z = max_z.max(i.maximum_z);
        }
        Aabb::from_min_max(Vec3::new(min_x,min_y, min_z), Vec3::new(max_x,max_y, max_z))
    }
}

#[derive(Clone, ShaderType)]
pub(crate) struct CuboidsTransform {
    pub matrix: Mat4,
    pub inv_matrix: Mat4,
}

impl CuboidsTransform {
    pub fn new(matrix: Mat4, inv_matrix: Mat4) -> Self {
        Self { matrix, inv_matrix }
    }

    pub fn from_matrix(m: Mat4) -> Self {
        Self::new(m, m.inverse())
    }

    pub fn position(&self) -> Vec3 {
        self.matrix.col(3).truncate()
    }
}

#[derive(Bundle)]
pub struct CuboidsBundle {
    pub color_options_id: ColorOptionsId,
    pub cuboids: Cuboids,
    #[bundle]
    pub spatial: SpatialBundle,
}
