// src/main.rs
mod vulkan;
mod scene;
mod game;
mod npc;
mod input;
mod ui;
mod gltf_loader;

use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, MouseButton, MouseScrollDelta},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::PhysicalSize,
};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, error};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();
    info!("Arnay Engine Editor with Vulkan");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Arnay Engine + AI NPC (Vulkan)")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    let app_state = Arc::new(Mutex::new(AppState::new(&window)));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                let mut state = app_state.lock();
                state.handle_window_event(&event);
                if let WindowEvent::CloseRequested = event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                let mut state = app_state.lock();
                state.update();
                state.render().unwrap();
            }
            _ => {}
        }
    });
}

struct AppState {
    vulkan: vulkan::VulkanContext,
    scene: scene::Scene,
    game: game::Game,
    input: input::InputManager,
    ui: ui::UIManager,
    gltf_loader: gltf_loader::GltfLoader,
    selected_object: Option<scene::ObjectId>,
    camera: scene::Camera,
    is_playing: bool,
    transform_mode: TransformMode,
}

#[derive(Clone, Copy, PartialEq)]
enum TransformMode {
    Translate,
    Rotate,
    Scale,
}

impl AppState {
    fn new(window: &Window) -> Self {
        let vulkan = vulkan::VulkanContext::new(window).unwrap();
        let mut scene = scene::Scene::new();
        let camera = scene::Camera::new(45.0, 0.1, 500.0);
        
        // Initialize game with default values
        let game = game::Game::new();
        let input = input::InputManager::new();
        let ui = ui::UIManager::new();
        let gltf_loader = gltf_loader::GltfLoader::new();

        let mut app = AppState {
            vulkan,
            scene,
            game,
            input,
            ui,
            gltf_loader,
            selected_object: None,
            camera,
            is_playing: false,
            transform_mode: TransformMode::Translate,
        };

        app.init_scene();
        app
    }

    fn init_scene(&mut self) {
        // Create initial objects
        let wall = self.scene.create_object(scene::ObjectType::Wall);
        let floor = self.scene.create_object(scene::ObjectType::Floor);
        let character = self.scene.create_object(scene::ObjectType::Character);
        let enemy = self.scene.create_object(scene::ObjectType::Enemy);
        let npc1 = self.scene.create_object(scene::ObjectType::NPC);
        let npc2 = self.scene.create_object(scene::ObjectType::NPC);

        // Position objects
        self.scene.set_position(wall, nalgebra::Vector3::new(0.0, 1.2, 0.0));
        self.scene.set_scale(wall, nalgebra::Vector3::new(3.0, 2.4, 0.2));
        
        self.scene.set_position(floor, nalgebra::Vector3::new(0.0, 0.075, 0.0));
        self.scene.set_scale(floor, nalgebra::Vector3::new(4.0, 0.15, 4.0));
        
        self.scene.set_position(character, nalgebra::Vector3::new(0.0, 0.5, 2.0));
        self.scene.set_scale(character, nalgebra::Vector3::new(0.8, 0.8, 0.8));
        
        self.scene.set_position(enemy, nalgebra::Vector3::new(3.0, 0.5, -2.0));
        self.scene.set_scale(enemy, nalgebra::Vector3::new(0.6, 0.6, 0.6));
        
        self.scene.set_position(npc1, nalgebra::Vector3::new(-1.2, 0.5, -1.5));
        self.scene.set_scale(npc1, nalgebra::Vector3::new(0.6, 0.6, 0.6));
        
        self.scene.set_position(npc2, nalgebra::Vector3::new(2.5, 0.5, 1.5));
        self.scene.set_scale(npc2, nalgebra::Vector3::new(0.6, 0.6, 0.6));

        // Set character as main
        self.game.set_character(character);
        self.scene.set_has_collision(character, true);
        self.scene.set_has_collision(enemy, true);
        self.scene.set_has_collision(npc1, true);
        self.scene.set_has_collision(npc2, true);

        // Setup NPC waypoints
        let npc1_waypoints = vec![
            nalgebra::Vector3::new(-1.2, 0.5, -1.5),
            nalgebra::Vector3::new(-2.5, 0.5, -3.0),
            nalgebra::Vector3::new(0.5, 0.5, -4.0),
            nalgebra::Vector3::new(2.0, 0.5, -2.0),
        ];
        self.scene.set_npc_waypoints(npc1, npc1_waypoints);

        let npc2_waypoints = vec![
            nalgebra::Vector3::new(2.5, 0.5, 1.5),
            nalgebra::Vector3::new(3.5, 0.5, 0.0),
            nalgebra::Vector3::new(2.0, 0.5, -1.0),
            nalgebra::Vector3::new(0.5, 0.5, 2.0),
        ];
        self.scene.set_npc_waypoints(npc2, npc2_waypoints);

        self.selected_object = Some(character);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        self.input.handle_window_event(event);
        
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                self.handle_keyboard(input);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_click(*state, *button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_mouse_move(*position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta);
            }
            _ => {}
        }
    }

    fn handle_keyboard(&mut self, input: &KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            let pressed = input.state == ElementState::Pressed;
            
            match key {
                winit::keyboard::VirtualKeyCode::Space => {
                    if pressed && self.is_playing {
                        self.game.jump();
                    }
                }
                winit::keyboard::VirtualKeyCode::W => self.input.set_key(input::Key::W, pressed),
                winit::keyboard::VirtualKeyCode::A => self.input.set_key(input::Key::A, pressed),
                winit::keyboard::VirtualKeyCode::S => self.input.set_key(input::Key::S, pressed),
                winit::keyboard::VirtualKeyCode::D => self.input.set_key(input::Key::D, pressed),
                winit::keyboard::VirtualKeyCode::Key1 => self.transform_mode = TransformMode::Translate,
                winit::keyboard::VirtualKeyCode::Key2 => self.transform_mode = TransformMode::Rotate,
                winit::keyboard::VirtualKeyCode::Key3 => self.transform_mode = TransformMode::Scale,
                winit::keyboard::VirtualKeyCode::Delete => {
                    if let Some(id) = self.selected_object {
                        self.scene.delete_object(id);
                        self.selected_object = None;
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_click(&mut self, state: ElementState, button: MouseButton) {
        if state == ElementState::Pressed && button == MouseButton::Left {
            // Raycast for object selection
            if let Some(id) = self.scene.raycast(&self.camera) {
                self.selected_object = Some(id);
            }
        }
    }

    fn handle_mouse_move(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.input.set_mouse_position(position.x as f32, position.y as f32);
    }

    fn handle_mouse_wheel(&mut self, delta: &MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.camera.zoom(*y as f32);
            }
            MouseScrollDelta::PixelDelta(pos) => {
                self.camera.zoom(pos.y as f32 * 0.1);
            }
        }
    }

    fn update(&mut self) {
        if self.is_playing {
            self.game.update(
                &mut self.scene,
                &self.input,
                1.0 / 60.0,
            );
            
            // Update camera to follow character
            if let Some(char_id) = self.game.character() {
                if let Some(pos) = self.scene.get_position(char_id) {
                    self.camera.follow(pos);
                }
            }
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get all renderable objects
        let objects: Vec<_> = self.scene
            .iter_objects()
            .map(|(id, obj)| {
                let pos = self.scene.get_position(id).unwrap_or_default();
                let rot = self.scene.get_rotation(id).unwrap_or_default();
                let scale = self.scene.get_scale(id).unwrap_or_default();
                let color = self.scene.get_color(id).unwrap_or([0.5, 0.5, 0.5, 1.0]);
                let is_selected = Some(id) == self.selected_object;
                
                scene::RenderObject {
                    position: pos,
                    rotation: rot,
                    scale: scale,
                    color: color,
                    object_type: obj.object_type,
                    is_selected,
                }
            })
            .collect();

        self.vulkan.render(&self.camera, &objects)?;
        Ok(())
    }
}
