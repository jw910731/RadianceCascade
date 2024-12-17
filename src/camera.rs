use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use egui::Direction;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct UniformCamera {
    matrix: Mat4,
    eye: Vec4,
}

impl UniformCamera {
    pub fn from_camera_project(camera: &Camera, projection: &Projection) -> Self {
        Self {
            eye: camera.position.extend(1.0),
            matrix: projection.calc_matrix() * camera.calc_matrix(),
        }
    }/*
    pub fn from_camera_project(camera: &Camera, projection: &Projection) -> Self {
        Self {
            eye: camera.position.extend(1.0),
            matrix: projection.calc_matrix() * camera.calc_matrix(),
        }
    }*/
}

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug, Clone, Default)]
pub struct Camera {
    pub position: glam::Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new<V: Into<glam::Vec3>>(position: V, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw,
            pitch,
        }
    }

    pub fn calc_matrix(&self) -> glam::Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        glam::Mat4::look_to_rh(
            self.position,
            glam::Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            glam::Vec3::Y,
        )
    }
}
#[derive(Debug, Clone, Default)]
pub enum Projection{

    Perspective( PerspectiveProjection ),
    Directional( DirectionalProjection ),
    #[default]
    Default
}
impl Projection {
    pub fn resize(&mut self, width: u32, height: u32) {
        match self {
            Projection::Perspective(s) => {s.resize( width, height)}
            Projection::Directional(s) => {s.resize( width, height)}
            Projection::Default=>{ }
        }
    }
    pub fn calc_matrix(&self) -> glam::Mat4{
        match self {
            Projection::Perspective(s) => {s.calc_matrix()}
            Projection::Directional(s) => {s.calc_matrix()}
            Projection::Default=>{ glam::Mat4::IDENTITY }
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct PerspectiveProjection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[derive(Debug, Clone, Default)]
pub struct DirectionalProjection {
    direction: Vec3,// normalize vector
    xleft: f32,
    xright: f32,
    ybottom: f32,
    ytop: f32,
    near_alone_dir: f32,
    far_alone_dir: f32,
}
impl PerspectiveProjection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.to_radians(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

impl DirectionalProjection {
    pub fn new(direction: Vec3,
               xleft: f32,
               xright: f32,
               ybottom: f32,
               ytop: f32,
               near_alone_dir: f32,
               far_alone_dir: f32,) -> Self {
        Self {
            direction,
            xleft,
            xright,
            ybottom,
            ytop,
            near_alone_dir,
            far_alone_dir,
        }
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
    }

    pub fn calc_matrix(&self) -> glam::Mat4 {
        let direction = self.direction.normalize();
        let near = self.near_alone_dir * direction.z;
        let far = self.far_alone_dir * direction.z;
        let a = 2.0 / (self.xright - self.xleft);
        let b = 2.0 / (self.ytop - self.ybottom);
        let c = -1.0 / (self.far_alone_dir - self.near_alone_dir);
        let delta_x = -a * ( direction.x / direction.z );
        let delta_y = -a * ( direction.y / direction.z );
        let tx = -(self.xright + self.xleft) / (self.xright - self.xleft);
        let ty = -(self.ytop + self.ybottom) / (self.ytop - self.ybottom);
        let tz = -(self.near_alone_dir) / (self.far_alone_dir - self.near_alone_dir);

        Mat4::from_cols(
            Vec4::new(a, 0.0, 0.0, 0.0),
            Vec4::new(0.0, b, 0.0, 0.0),
            Vec4::new(delta_x, delta_y, c, 0.0),
            Vec4::new(tx, ty, tz, 1.0),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(
        &mut self,
        physical_key: &PhysicalKey,
        logical_key: &Key,
        state: ElementState,
    ) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match logical_key {
            Key::Named(NamedKey::Space) => {
                self.amount_up = amount;
                return true;
            }
            _ => {}
        }
        match physical_key {
            PhysicalKey::Code(KeyCode::ShiftLeft) => {
                self.amount_down = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyW) => {
                self.amount_forward = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                self.amount_left = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                self.amount_backward = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                self.amount_right = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // 假定一行为 100 个像素，你可以随意修改这个值
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = glam::Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = glam::Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward =
            glam::Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // 旋转
        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}
