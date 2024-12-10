use std::borrow::Borrow;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec4, Vec3, Vec4};
use winit::event::WindowEvent;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct UniformCamera {
    matrix: Mat4,
    eye: Vec4,
    enable_normal_map: UVec4,
}

impl<T> From<T> for UniformCamera
where
    T: Borrow<Camera>,
{
    fn from(value: T) -> Self {
        Self {
            matrix: value.borrow().get_view_project(),
            eye: (value.borrow().get_view_position(), 1.0).into(),
            enable_normal_map: UVec4::new((value.borrow().enable_normal_map).into(), 0, 0, 0),
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
    delta: Vec3,
    eye_rotation_vertical: f32,
    eye_rotation_horizontal: f32,
    distance_to_axis: f32,
    enable_normal_map: bool,
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
        let eye = self.get_view_position();
        let target = self.target + self.delta;
        let view = Mat4::look_at_lh(eye, target, self.up);
        let proj = Mat4::perspective_lh(self.fovy, self.aspect, self.znear, self.zfar);
        return proj * view;
    }
    pub fn get_view_position(&self) -> Vec3 {
        (Mat4::from_axis_angle(self.up, self.eye_rotation_horizontal)
            .mul_mat4(&Mat4::from_axis_angle(
                self.up.cross(self.eye - self.target).normalize(),
                self.eye_rotation_vertical,
            ))
            .project_point3(self.eye - self.target))
            * self.distance_to_axis
            + self.target
            + self.delta
    }

    pub fn process_events(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(
        &mut self,
        eye_rotation_horizontal: f32,
        eye_rotation_vertical: f32,
        delta: Vec3,
        eye_pos_distance: f32,
        enable_normal_map: bool,
    ) {
        self.eye_rotation_horizontal = eye_rotation_horizontal;
        self.eye_rotation_vertical = eye_rotation_vertical;
        self.delta = delta;
        self.distance_to_axis = eye_pos_distance;
        self.enable_normal_map = enable_normal_map;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}
