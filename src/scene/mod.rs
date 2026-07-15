// src/scene/mod.rs
use nalgebra::{Vector3, Matrix4, Quaternion};
use slotmap::{SlotMap, Key};
use std::collections::HashMap;
use rand::Rng;

type ObjectIdKey = slotmap::DefaultKey;
pub type ObjectId = ObjectIdKey;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjectType {
    Cube,
    Sphere,
    Cylinder,
    Torus,
    Wall,
    Floor,
    Beam,
    StairStep,
    Character,
    Enemy,
    NPC,
    Camera,
    GLBModel,
}

#[derive(Clone)]
pub struct Object {
    pub object_type: ObjectType,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub color: [f32; 4],
    pub has_collision: bool,
    pub is_selected: bool,
    pub speed: f32,
    pub jump_height: f32,
    pub hp: f32,
    pub damage: f32,
    pub velocity: Vector3<f32>,
    pub is_grounded: bool,
    pub jump_count: u32,
    pub npc_state: NPCState,
    pub npc_waypoints: Vec<Vector3<f32>>,
    pub npc_current_target: usize,
    pub npc_talk_timer: f32,
    pub npc_is_waiting: bool,
    pub npc_wait_timer: f32,
    pub texture_name: Option<String>,
    pub source_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NPCState {
    Patrol,
    Idle,
    Talk,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            object_type: ObjectType::Cube,
            position: Vector3::zeros(),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            color: [0.5, 0.5, 0.5, 1.0],
            has_collision: false,
            is_selected: false,
            speed: 4.0,
            jump_height: 8.0,
            hp: 100.0,
            damage: 10.0,
            velocity: Vector3::zeros(),
            is_grounded: true,
            jump_count: 0,
            npc_state: NPCState::Patrol,
            npc_waypoints: Vec::new(),
            npc_current_target: 0,
            npc_talk_timer: 0.0,
            npc_is_waiting: false,
            npc_wait_timer: 0.0,
            texture_name: None,
            source_name: None,
        }
    }
}

pub struct Scene {
    objects: SlotMap<ObjectIdKey, Object>,
    character: Option<ObjectId>,
    cameras: Vec<ObjectId>,
    npcs: Vec<ObjectId>,
    enemies: Vec<ObjectId>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: SlotMap::new(),
            character: None,
            cameras: Vec::new(),
            npcs: Vec::new(),
            enemies: Vec::new(),
        }
    }

    pub fn create_object(&mut self, object_type: ObjectType) -> ObjectId {
        let mut obj = Object::default();
        obj.object_type = object_type;
        obj.color = Self::get_default_color(object_type);
        
        let id = self.objects.insert(obj);
        
        match object_type {
            ObjectType::Character => {
                self.character = Some(id);
            }
            ObjectType::NPC => {
                self.npcs.push(id);
            }
            ObjectType::Enemy => {
                self.enemies.push(id);
            }
            ObjectType::Camera => {
                self.cameras.push(id);
            }
            _ => {}
        }
        
        id
    }

    fn get_default_color(object_type: ObjectType) -> [f32; 4] {
        match object_type {
            ObjectType::Cube => [0.49, 0.78, 0.89, 1.0],
            ObjectType::Sphere => [0.42, 0.69, 0.84, 1.0],
            ObjectType::Cylinder => [0.54, 0.83, 0.63, 1.0],
            ObjectType::Torus => [0.91, 0.72, 0.42, 1.0],
            ObjectType::Character => [0.27, 0.87, 0.53, 1.0],
            ObjectType::Enemy => [0.87, 0.27, 0.27, 1.0],
            ObjectType::NPC => [0.60, 0.40, 0.80, 1.0],
            ObjectType::Wall => [0.72, 0.68, 0.63, 1.0],
            ObjectType::Floor => [0.60, 0.60, 0.66, 1.0],
            ObjectType::Beam => [0.79, 0.63, 0.42, 1.0],
            ObjectType::StairStep => [0.66, 0.56, 0.48, 1.0],
            ObjectType::Camera => [0.16, 0.29, 0.35, 1.0],
            ObjectType::GLBModel => [0.5, 0.5, 0.5, 1.0],
        }
    }

    pub fn delete_object(&mut self, id: ObjectId) {
        if let Some(obj) = self.objects.get(id) {
            match obj.object_type {
                ObjectType::Character => {
                    self.character = None;
                }
                ObjectType::NPC => {
                    self.npcs.retain(|&npc_id| npc_id != id);
                }
                ObjectType::Enemy => {
                    self.enemies.retain(|&enemy_id| enemy_id != id);
                }
                ObjectType::Camera => {
                    self.cameras.retain(|&cam_id| cam_id != id);
                }
                _ => {}
            }
        }
        self.objects.remove(id);
    }

    pub fn get_object(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(id)
    }

    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(id)
    }

    pub fn iter_objects(&self) -> impl Iterator<Item = (ObjectId, &Object)> {
        self.objects.iter().map(|(id, obj)| (id, obj))
    }

    pub fn get_position(&self, id: ObjectId) -> Option<Vector3<f32>> {
        self.objects.get(id).map(|o| o.position)
    }

    pub fn set_position(&mut self, id: ObjectId, pos: Vector3<f32>) {
        if let Some(obj) = self.objects.get_mut(id) {
            obj.position = pos;
        }
    }

    pub fn get_rotation(&self, id: ObjectId) -> Option<Quaternion<f32>> {
        self.objects.get(id).map(|o| o.rotation)
    }

    pub fn set_rotation(&mut self, id: ObjectId, rot: Quaternion<f32>) {
        if let Some(obj) = self.objects.get_mut(id) {
            obj.rotation = rot;
        }
    }

    pub fn get_scale(&self, id: ObjectId) -> Option<Vector3<f32>> {
        self.objects.get(id).map(|o| o.scale)
    }

    pub fn set_scale(&mut self, id: ObjectId, scale: Vector3<f32>) {
        if let Some(obj) = self.objects.get_mut(id) {
            obj.scale = scale;
        }
    }

    pub fn get_color(&self, id: ObjectId) -> Option<[f32; 4]> {
        self.objects.get(id).map(|o| o.color)
    }

    pub fn set_color(&mut self, id: ObjectId, color: [f32; 4]) {
        if let Some(obj) = self.objects.get_mut(id) {
            obj.color = color;
        }
    }

    pub fn set_has_collision(&mut self, id: ObjectId, has_collision: bool) {
        if let Some(obj) = self.objects.get_mut(id) {
            obj.has_collision = has_collision;
        }
    }

    pub fn get_has_collision(&self, id: ObjectId) -> bool {
        self.objects.get(id).map(|o| o.has_collision).unwrap_or(false)
    }

    pub fn set_npc_waypoints(&mut self, id: ObjectId, waypoints: Vec<Vector3<f32>>) {
        if let Some(obj) = self.objects.get_mut(id) {
            obj.npc_waypoints = waypoints;
        }
    }

    pub fn character(&self) -> Option<ObjectId> {
        self.character
    }

    pub fn npcs(&self) -> &Vec<ObjectId> {
        &self.npcs
    }

    pub fn enemies(&self) -> &Vec<ObjectId> {
        &self.enemies
    }

    pub fn raycast(&self, camera: &Camera) -> Option<ObjectId> {
        // Simplified raycast - in real implementation would use a proper picker
        // For now, return the first object found
        self.objects.iter().next().map(|(id, _)| id)
    }
}

pub struct Camera {
    position: Vector3<f32>,
    target: Vector3<f32>,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Camera {
            position: Vector3::new(6.0, 4.0, 8.0),
            target: Vector3::zeros(),
            fov,
            aspect: 16.0 / 9.0,
            near,
            far,
            distance: 10.0,
            yaw: 0.0,
            pitch: 0.5,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let eye = self.position;
        let target = self.target;
        let up = Vector3::new(0.0, 1.0, 0.0);
        Matrix4::look_at_rh(&eye, &target, &up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.aspect, self.fov * std::f32::consts::PI / 180.0, self.near, self.far)
    }

    pub fn follow(&mut self, target: Vector3<f32>) {
        self.target = target;
        // Calculate position based on orbit
        let distance = self.distance;
        let x = target.x + distance * self.yaw.cos() * self.pitch.cos();
        let y = target.y + distance * self.pitch.sin();
        let z = target.z + distance * self.yaw.sin() * self.pitch.cos();
        self.position = Vector3::new(x, y, z);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta * 0.5).max(0.5).min(50.0);
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.005;
        self.pitch = (self.pitch + dy * 0.005).clamp(0.1, 1.4);
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

pub struct RenderObject {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub color: [f32; 4],
    pub object_type: ObjectType,
    pub is_selected: bool,
}
