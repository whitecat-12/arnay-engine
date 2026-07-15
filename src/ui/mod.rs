// src/ui/mod.rs
pub struct UIManager {
    // UI state
    selected_object_name: String,
    fps: u32,
    object_count: u32,
    enemy_count: u32,
    npc_count: u32,
    is_playing: bool,
}

impl UIManager {
    pub fn new() -> Self {
        UIManager {
            selected_object_name: String::from("None"),
            fps: 60,
            object_count: 0,
            enemy_count: 0,
            npc_count: 0,
            is_playing: false,
        }
    }

    pub fn update(
        &mut self,
        selected_name: &str,
        fps: u32,
        object_count: u32,
        enemy_count: u32,
        npc_count: u32,
        is_playing: bool,
    ) {
        self.selected_object_name = selected_name.to_string();
        self.fps = fps;
        self.object_count = object_count;
        self.enemy_count = enemy_count;
        self.npc_count = npc_count;
        self.is_playing = is_playing;
    }

    // UI rendering would use Vulkan or a 2D overlay
    // For now, this is a placeholder
}
