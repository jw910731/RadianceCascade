use std::borrow::Borrow;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use winit::event::WindowEvent;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct UniformCamera {
    matrix: Mat4,
    eye: Vec4,
}

impl<T> From<T> for UniformCamera
where
    T: Borrow<Camera>,
{
    fn from(value: T) -> Self {
        Self {
            matrix: value.borrow().get_view_project(),
            eye: (value.borrow().eye, 1.0).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    delta_y: f32,
    eye_rotation: f32,
    distance_to_axis: f32,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            ..Default::default()
        }
    }

    pub fn get_view_project(&self) -> Mat4 {
        let target = self.target + Vec3::ZERO.with_y(self.delta_y);
        let eye = ( Mat4::from_axis_angle(self.up, self.eye_rotation)
            .project_point3(self.eye.with_y(0.0) - self.target)
            + Vec3::ZERO.with_y(self.eye.y)
            ) * self.distance_to_axis
            + self.target;
        let view = Mat4::look_at_lh(eye, target, self.up);
        let proj = Mat4::perspective_lh(self.fovy, self.aspect, self.znear, self.zfar);
        return proj * view;
    }

    pub fn process_events(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self, eye_pos_rotation: f32, look_at_y: f32, eye_pos_distance: f32) {
        self.eye_rotation = eye_pos_rotation;
        self.delta_y = look_at_y;
        self.distance_to_axis = eye_pos_distance;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}
