use std::borrow::Borrow;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

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

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    eye: Vec3,
    direction: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    camera_control: CameraController,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        direction: Vec3,
        up: Vec3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
        speed: f32,
    ) -> Self {
        Self {
            eye,
            direction,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            camera_control: CameraController::new(speed),
        }
    }

    pub fn get_view_project(&self) -> Mat4 {
        let view = Mat4::look_to_lh(self.eye, self.direction, self.up);
        let proj = Mat4::perspective_lh(self.fovy, self.aspect, self.znear, self.zfar);
        return proj * view;
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        self.camera_control.process_events(event)
    }

    pub fn update(&mut self) {
        (self.eye, self.direction) = self.camera_control.update_camera(self);
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_polar_positive_pressed: bool,
    is_polar_negative_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyR => {
                        self.is_polar_positive_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyF => {
                        self.is_polar_negative_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &Camera) -> (Vec3, Vec3) {
        let mut camera = camera.clone();
        let forward = camera.direction - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.direction - camera.eye;
        let forward_mag = forward.length();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye =
                camera.direction - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye =
                camera.direction - (forward - right * self.speed).normalize() * forward_mag;
        }

        if self.is_polar_positive_pressed {
            camera.direction += camera.up * self.speed * 0.1;
        }
        if self.is_polar_negative_pressed {
            camera.direction -= camera.up * self.speed * 0.1;
        }
        (camera.eye, camera.direction)
    }
}
