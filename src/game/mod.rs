// src/game/mod.rs
use crate::scene::{Scene, ObjectId, ObjectType, NPCState};
use crate::input::InputManager;
use nalgebra::Vector3;
use rand::Rng;

const GRAVITY: f32 = -25.0;
const MAX_JUMPS: u32 = 2;
const ENEMY_CHASE_RANGE: f32 = 8.0;
const NPC_TALK_RADIUS: f32 = 3.0;
const NPC_MESSAGES: [&str; 15] = [
    "Halo, petualang!",
    "Selamat datang di dunia!",
    "Aku NPC yang ramah 😊",
    "Ada apa?",
    "Jangan lupa istirahat!",
    "Hari ini cerah sekali ☀️",
    "Aku suka jalan-jalan.",
    "Pernah ke gunung?",
    "Hati-hati di jalan!",
    "Senang bertemu denganmu!",
    "Apa kabar?",
    "Semoga harimu menyenangkan!",
    "Ada misi baru?",
    "Aku menjaga desa ini.",
    "Selamat berpetualang!",
];

pub struct Game {
    character: Option<ObjectId>,
    is_playing: bool,
    npc_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        Game {
            character: None,
            is_playing: false,
            npc_timer: 0.0,
        }
    }

    pub fn set_character(&mut self, id: ObjectId) {
        self.character = Some(id);
    }

    pub fn character(&self) -> Option<ObjectId> {
        self.character
    }

    pub fn jump(&mut self) {
        if let Some(id) = self.character {
            if let Some(obj) = self.scene.get_object_mut(id) {
                if obj.is_grounded {
                    obj.velocity.y = obj.jump_height;
                    obj.is_grounded = false;
                    obj.jump_count = 1;
                } else if obj.jump_count < MAX_JUMPS {
                    obj.velocity.y = obj.jump_height * 0.85;
                    obj.jump_count += 1;
                }
            }
        }
    }

    pub fn update(&mut self, scene: &mut Scene, input: &InputManager, delta: f32) {
        if !self.is_playing {
            return;
        }

        let character_id = match self.character {
            Some(id) => id,
            None => return,
        };

        // Get character
        let character = match scene.get_object_mut(character_id) {
            Some(c) => c,
            None => return,
        };

        // Movement
        let mut move_dir = Vector3::zeros();
        if input.is_key_pressed(crate::input::Key::W) { move_dir.z -= 1.0; }
        if input.is_key_pressed(crate::input::Key::S) { move_dir.z += 1.0; }
        if input.is_key_pressed(crate::input::Key::A) { move_dir.x -= 1.0; }
        if input.is_key_pressed(crate::input::Key::D) { move_dir.x += 1.0; }

        let move_len = move_dir.norm();
        if move_len > 0.0 {
            move_dir /= move_len;
            let speed = character.speed;
            character.velocity.x = move_dir.x * speed;
            character.velocity.z = move_dir.z * speed;
        } else {
            character.velocity.x *= 0.9;
            character.velocity.z *= 0.9;
        }

        // Apply gravity
        character.velocity.y += GRAVITY * delta;

        // Update position
        character.position.x += character.velocity.x * delta;
        character.position.y += character.velocity.y * delta;
        character.position.z += character.velocity.z * delta;

        // Ground check
        let ground_y = 0.5;
        if character.position.y < ground_y {
            character.position.y = ground_y;
            character.velocity.y = 0.0;
            if !character.is_grounded {
                character.is_grounded = true;
                character.jump_count = 0;
            }
        } else {
            if character.is_grounded && character.velocity.y > 0.0 {
                character.is_grounded = false;
                character.jump_count = 1;
            }
        }

        // Rotate character towards movement direction
        if move_len > 0.0 {
            let target_angle = move_dir.x.atan2(-move_dir.z);
            let mut current_angle = character.rotation.euler_angles().1;
            let mut diff = target_angle - current_angle;
            while diff > std::f32::consts::PI { diff -= std::f32::consts::PI * 2.0; }
            while diff < -std::f32::consts::PI { diff += std::f32::consts::PI * 2.0; }
            let rot_quat = nalgebra::Quaternion::from_axis_angle(
                &nalgebra::Vector3::y_axis(),
                diff * 0.1,
            );
            character.rotation = rot_quat * character.rotation;
        }

        // Enemy AI
        let char_pos = character.position;
        for enemy_id in scene.enemies().clone() {
            if let Some(enemy) = scene.get_object_mut(enemy_id) {
                let dx = char_pos.x - enemy.position.x;
                let dz = char_pos.z - enemy.position.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist < ENEMY_CHASE_RANGE && dist > 0.3 {
                    let speed = enemy.speed;
                    let step_x = (dx / dist) * speed * delta;
                    let step_z = (dz / dist) * speed * delta;
                    enemy.position.x += step_x;
                    enemy.position.z += step_z;

                    let target_angle = dx.atan2(dz);
                    let mut current_angle = enemy.rotation.euler_angles().1;
                    let mut diff = target_angle - current_angle;
                    while diff > std::f32::consts::PI { diff -= std::f32::consts::PI * 2.0; }
                    while diff < -std::f32::consts::PI { diff += std::f32::consts::PI * 2.0; }
                    let rot_quat = nalgebra::Quaternion::from_axis_angle(
                        &nalgebra::Vector3::y_axis(),
                        diff * 0.05,
                    );
                    enemy.rotation = rot_quat * enemy.rotation;
                }

                if enemy.position.y < 0.5 {
                    enemy.position.y = 0.5;
                }
            }
        }

        // NPC AI (update at interval)
        self.npc_timer += delta;
        if self.npc_timer >= 0.05 {
            self.update_npcs(scene, delta);
            self.npc_timer = 0.0;
        }

        // Collision
        self.resolve_collisions(scene);
    }

    fn update_npcs(&mut self, scene: &mut Scene, delta: f32) {
        let char_pos = match self.character {
            Some(id) => scene.get_position(id).unwrap_or_default(),
            None => return,
        };

        let npc_ids = scene.npcs().clone();
        for npc_id in npc_ids {
            if let Some(npc) = scene.get_object_mut(npc_id) {
                let dx = char_pos.x - npc.position.x;
                let dz = char_pos.z - npc.position.z;
                let dist = (dx * dx + dz * dz).sqrt();

                match npc.npc_state {
                    NPCState::Talk => {
                        npc.npc_talk_timer -= delta;
                        if npc.npc_talk_timer <= 0.0 {
                            npc.npc_state = NPCState::Patrol;
                            npc.npc_is_waiting = true;
                            npc.npc_wait_timer = 1.5 + rand::thread_rng().gen_range(0.0..2.0);
                            // Hide chat bubble
                        }
                    }
                    NPCState::Idle => {
                        // Waiting after talking
                    }
                    NPCState::Patrol => {
                        // Check if character is near for talking
                        if dist < NPC_TALK_RADIUS && !npc.npc_is_waiting {
                            npc.npc_state = NPCState::Talk;
                            npc.npc_talk_timer = 3.5;
                            // Show chat message
                            let msg = NPC_MESSAGES[rand::thread_rng().gen_range(0..NPC_MESSAGES.len())];
                            // Show bubble
                        }

                        // Patrol logic
                        let waypoints = &npc.npc_waypoints;
                        if waypoints.is_empty() {
                            continue;
                        }

                        let target_idx = npc.npc_current_target % waypoints.len();
                        let target = waypoints[target_idx];
                        let tx = target.x - npc.position.x;
                        let tz = target.z - npc.position.z;
                        let t_dist = (tx * tx + tz * tz).sqrt();

                        if t_dist < 0.6 {
                            npc.npc_current_target = (target_idx + 1) % waypoints.len();
                            npc.npc_is_waiting = true;
                            npc.npc_wait_timer = 0.5 + rand::thread_rng().gen_range(0.0..1.5);
                        } else {
                            let spd = npc.speed;
                            let step_x = (tx / t_dist) * spd * delta * 1.5;
                            let step_z = (tz / t_dist) * spd * delta * 1.5;
                            npc.position.x += step_x;
                            npc.position.z += step_z;

                            // Rotate towards target
                            let target_angle = tx.atan2(tz);
                            let mut current_angle = npc.rotation.euler_angles().1;
                            let mut diff = target_angle - current_angle;
                            while diff > std::f32::consts::PI { diff -= std::f32::consts::PI * 2.0; }
                            while diff < -std::f32::consts::PI { diff += std::f32::consts::PI * 2.0; }
                            let rot_quat = nalgebra::Quaternion::from_axis_angle(
                                &nalgebra::Vector3::y_axis(),
                                diff * 0.08,
                            );
                            npc.rotation = rot_quat * npc.rotation;
                        }

                        if npc.position.y < 0.5 {
                            npc.position.y = 0.5;
                        }
                    }
                }

                // Handle waiting
                if npc.npc_is_waiting {
                    npc.npc_wait_timer -= delta;
                    if npc.npc_wait_timer <= 0.0 {
                        npc.npc_is_waiting = false;
                        if npc.npc_state == NPCState::Idle {
                            npc.npc_state = NPCState::Patrol;
                        }
                    }
                }
            }
        }
    }

    fn resolve_collisions(&mut self, scene: &mut Scene) {
        // Simplified collision resolution
        // In a real implementation, this would use proper physics
        let moving_objects: Vec<ObjectId> = scene
            .iter_objects()
            .filter(|(_, obj)| {
                obj.object_type == ObjectType::Character ||
                obj.object_type == ObjectType::Enemy ||
                obj.object_type == ObjectType::NPC
            })
            .map(|(id, _)| id)
            .collect();

        let static_objects: Vec<ObjectId> = scene
            .iter_objects()
            .filter(|(_, obj)| {
                !moving_objects.contains(&id) &&
                obj.has_collision &&
                obj.object_type != ObjectType::Camera
            })
            .map(|(id, _)| id)
            .collect();

        // Resolve collisions between moving and static objects
        for moving_id in &moving_objects {
            for static_id in &static_objects {
                if moving_id == static_id { continue; }
                // Simple push-out collision
                // In real implementation, would use proper box/sphere collision
            }
        }
    }

    pub fn start(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}
