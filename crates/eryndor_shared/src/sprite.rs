//! Sprite and animation data types shared between editor and game
//!
//! Provides data structures for defining sprites with spritesheet-based animations,
//! and utilities for creating Bevy components from this data.

use bevy::prelude::*;
use bevy::sprite::{Anchor, Sprite};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Animation loop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Reflect)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    #[default]
    Loop,
    Once,
    PingPong,
}

impl LoopMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            LoopMode::Loop => "Loop",
            LoopMode::Once => "Once",
            LoopMode::PingPong => "Ping-Pong",
        }
    }

    pub fn all() -> &'static [LoopMode] {
        &[LoopMode::Loop, LoopMode::Once, LoopMode::PingPong]
    }
}

/// A single animation definition
#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect)]
pub struct AnimationDef {
    /// Frame indices into the spritesheet grid (left-to-right, top-to-bottom)
    pub frames: Vec<usize>,
    /// Duration of each frame in milliseconds
    #[serde(default = "default_frame_duration")]
    pub frame_duration_ms: u32,
    /// How the animation loops
    #[serde(default)]
    pub loop_mode: LoopMode,
}

fn default_frame_duration() -> u32 {
    100
}

impl AnimationDef {
    /// Get the frame duration as a Duration
    pub fn frame_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.frame_duration_ms as u64)
    }
}

/// Sprite data with spritesheet reference and animations
#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect)]
pub struct SpriteData {
    /// Path to the spritesheet image (relative to assets)
    pub sheet_path: String,
    /// Width of each frame in pixels
    pub frame_width: u32,
    /// Height of each frame in pixels
    pub frame_height: u32,
    /// Number of columns in the spritesheet
    #[serde(default)]
    pub columns: u32,
    /// Number of rows in the spritesheet
    #[serde(default)]
    pub rows: u32,
    /// Pivot point X (0.0-1.0, where 0.5 is center)
    #[serde(default = "default_pivot")]
    pub pivot_x: f32,
    /// Pivot point Y (0.0-1.0, where 0.5 is center)
    #[serde(default = "default_pivot")]
    pub pivot_y: f32,
    /// Named animations
    #[serde(default)]
    #[reflect(ignore)]
    pub animations: HashMap<String, AnimationDef>,
}

fn default_pivot() -> f32 {
    0.5
}

impl SpriteData {
    /// Get total frame count based on grid
    pub fn total_frames(&self) -> usize {
        (self.columns * self.rows) as usize
    }

    /// Convert frame index to grid position (col, row)
    pub fn frame_to_grid(&self, frame: usize) -> (u32, u32) {
        if self.columns == 0 {
            return (0, 0);
        }
        let col = (frame as u32) % self.columns;
        let row = (frame as u32) / self.columns;
        (col, row)
    }

    /// Convert grid position to frame index
    pub fn grid_to_frame(&self, col: u32, row: u32) -> usize {
        (row * self.columns + col) as usize
    }

    /// Get pixel rect for a frame: (x, y, width, height)
    pub fn frame_rect(&self, frame: usize) -> (u32, u32, u32, u32) {
        let (col, row) = self.frame_to_grid(frame);
        (
            col * self.frame_width,
            row * self.frame_height,
            self.frame_width,
            self.frame_height,
        )
    }

    /// Get the frame size as UVec2
    pub fn frame_size(&self) -> UVec2 {
        UVec2::new(self.frame_width, self.frame_height)
    }

    /// Get the anchor point based on pivot
    pub fn anchor(&self) -> Anchor {
        Anchor(Vec2::new(
            self.pivot_x - 0.5,
            0.5 - self.pivot_y,
        ))
    }

    /// Create a TextureAtlasLayout from this sprite data
    pub fn create_atlas_layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(
            self.frame_size(),
            self.columns,
            self.rows,
            None,
            None,
        )
    }
}

/// Component for animating sprites using editor-defined animations
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpriteAnimator {
    /// Name of the current animation
    pub current_animation: String,
    /// Current frame index within the animation
    pub frame_index: usize,
    /// Time accumulated since last frame change
    pub timer: f32,
    /// Whether animation is playing
    pub playing: bool,
    /// Ping-pong direction (true = forward, false = backward)
    pub ping_pong_forward: bool,
    /// Set to true when the animation reaches its end (for Once mode)
    pub finished: bool,
}

impl SpriteAnimator {
    /// Create a new animator starting at the given animation
    pub fn new(animation_name: impl Into<String>) -> Self {
        Self {
            current_animation: animation_name.into(),
            frame_index: 0,
            timer: 0.0,
            playing: true,
            ping_pong_forward: true,
            finished: false,
        }
    }

    /// Play the specified animation from the beginning
    pub fn play(&mut self, animation_name: impl Into<String>) {
        self.current_animation = animation_name.into();
        self.frame_index = 0;
        self.timer = 0.0;
        self.playing = true;
        self.ping_pong_forward = true;
        self.finished = false;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// Check if currently playing the specified animation
    pub fn is_playing(&self, animation_name: &str) -> bool {
        self.playing && self.current_animation == animation_name
    }
}

/// Resource containing sprite data definitions loaded from editor JSON
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SpriteDefinitions {
    /// Map of entity type name to sprite data
    #[reflect(ignore)]
    pub sprites: HashMap<String, SpriteData>,
}

impl SpriteDefinitions {
    /// Get sprite data by entity type name
    pub fn get(&self, name: &str) -> Option<&SpriteData> {
        self.sprites.get(name)
    }

    /// Insert sprite data for an entity type
    pub fn insert(&mut self, name: String, data: SpriteData) {
        self.sprites.insert(name, data);
    }
}

/// Bundle for spawning animated sprites
#[derive(Bundle, Default)]
pub struct AnimatedSpriteBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub animator: SpriteAnimator,
}

/// System to animate sprites based on SpriteAnimator component
pub fn animate_sprites(
    time: Res<Time>,
    definitions: Option<Res<SpriteDefinitions>>,
    mut query: Query<(&mut SpriteAnimator, &mut Sprite, Option<&Name>)>,
) {
    let Some(defs) = definitions else { return };

    for (mut animator, mut sprite, name) in &mut query {
        if !animator.playing || animator.finished {
            continue;
        }

        // Get sprite data - try Name component first, then current_animation as fallback
        let sprite_data = name
            .and_then(|n| defs.get(n.as_str()))
            .or_else(|| defs.get(&animator.current_animation));

        let Some(sprite_data) = sprite_data else {
            continue;
        };

        let Some(anim) = sprite_data.animations.get(&animator.current_animation) else {
            continue;
        };

        if anim.frames.is_empty() {
            continue;
        }

        // Update timer
        animator.timer += time.delta_secs() * 1000.0;
        let frame_duration = anim.frame_duration_ms as f32;

        if animator.timer >= frame_duration {
            animator.timer -= frame_duration;

            // Advance frame based on loop mode
            match anim.loop_mode {
                LoopMode::Loop => {
                    animator.frame_index = (animator.frame_index + 1) % anim.frames.len();
                }
                LoopMode::Once => {
                    if animator.frame_index < anim.frames.len() - 1 {
                        animator.frame_index += 1;
                    } else {
                        animator.finished = true;
                    }
                }
                LoopMode::PingPong => {
                    if animator.ping_pong_forward {
                        if animator.frame_index < anim.frames.len() - 1 {
                            animator.frame_index += 1;
                        } else {
                            animator.ping_pong_forward = false;
                            if animator.frame_index > 0 {
                                animator.frame_index -= 1;
                            }
                        }
                    } else if animator.frame_index > 0 {
                        animator.frame_index -= 1;
                    } else {
                        animator.ping_pong_forward = true;
                        animator.frame_index += 1;
                    }
                }
            }
        }

        // Update the sprite's texture atlas index
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if animator.frame_index < anim.frames.len() {
                atlas.index = anim.frames[animator.frame_index];
            }
        }
    }
}

/// Plugin for sprite animation system
pub struct SpriteAnimationPlugin;

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LoopMode>()
            .register_type::<AnimationDef>()
            .register_type::<SpriteData>()
            .register_type::<SpriteAnimator>()
            .register_type::<SpriteDefinitions>()
            .init_resource::<SpriteDefinitions>()
            .add_systems(Update, animate_sprites);
    }
}
