// src/npc/mod.rs
use crate::scene::{Scene, ObjectId};
use nalgebra::Vector3;

pub struct NPCAgent {
    id: ObjectId,
    state: NPCState,
    waypoints: Vec<Vector3<f32>>,
    current_target: usize,
    talk_timer: f32,
    is_waiting: bool,
    wait_timer: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NPCState {
    Patrol,
    Idle,
    Talk,
}

impl NPCAgent {
    pub fn new(id: ObjectId, waypoints: Vec<Vector3<f32>>) -> Self {
        NPCAgent {
            id,
            state: NPCState::Patrol,
            waypoints,
            current_target: 0,
            talk_timer: 0.0,
            is_waiting: false,
            wait_timer: 0.0,
        }
    }

    pub fn update(&mut self, scene: &mut Scene, char_pos: Vector3<f32>, delta: f32) {
        let npc = match scene.get_object_mut(self.id) {
            Some(n) => n,
            None => return,
        };

        let dx = char_pos.x - npc.position.x;
        let dz = char_pos.z - npc.position.z;
        let dist = (dx * dx + dz * dz).sqrt();

        match self.state {
            NPCState::Talk => {
                self.talk_timer -= delta;
                if self.talk_timer <= 0.0 {
                    self.state = NPCState::Patrol;
                    self.is_waiting = true;
                    self.wait_timer = 1.5 + rand::random::<f32>() * 2.0;
                }
            }
            NPCState::Idle => {
                // Just waiting
            }
            NPCState::Patrol => {
                // Check if character is near
                if dist < 3.0 && !self.is_waiting {
                    self.state = NPCState::Talk;
                    self.talk_timer = 3.5;
                    // Show chat bubble
                }

                // Patrol logic
                if self.waypoints.is_empty() {
                    return;
                }

                let target_idx = self.current_target % self.waypoints.len();
                let target = self.waypoints[target_idx];
                let tx = target.x - npc.position.x;
                let tz = target.z - npc.position.z;
                let t_dist = (tx * tx + tz * tz).sqrt();

                if t_dist < 0.6 {
                    self.current_target = (target_idx + 1) % self.waypoints.len();
                    self.is_waiting = true;
                    self.wait_timer = 0.5 + rand::random::<f32>() * 1.5;
                } else {
                    let spd = npc.speed;
                    let step_x = (tx / t_dist) * spd * delta * 1.5;
                    let step_z = (tz / t_dist) * spd * delta * 1.5;
                    npc.position.x += step_x;
                    npc.position.z += step_z;
                }
            }
        }

        // Handle waiting
        if self.is_waiting {
            self.wait_timer -= delta;
            if self.wait_timer <= 0.0 {
                self.is_waiting = false;
                if self.state == NPCState::Idle {
                    self.state = NPCState::Patrol;
                }
            }
        }
    }
}
