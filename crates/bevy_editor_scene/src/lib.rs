//! Scene editor module for managing game entities (separate from tilemap)
//! Uses Bevy's DynamicScene for serialization/deserialization

use bevy::prelude::*;
use bevy::scene::{DynamicScene, DynamicSceneBuilder, DynamicSceneRoot};
use bevy_editor_formats::{BevyScene, LevelData};
use std::path::Path;

/// Marker component for entities that are part of the edited scene
/// (not editor UI elements)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct EditorSceneEntity;

/// Resource managing the currently edited scene
#[derive(Resource, Default)]
pub struct EditorScene {
    /// Root entity that all scene entities are parented to
    pub root_entity: Option<Entity>,
    /// Currently selected entity in the scene tree
    pub selected_entity: Option<Entity>,
    /// Whether the scene has unsaved changes
    pub is_modified: bool,
}

impl EditorScene {
    /// Create a new empty scene with a root entity
    pub fn new(commands: &mut Commands) -> Self {
        let root_entity = commands
            .spawn((
                Name::new("Scene Root"),
                Transform::default(),
                Visibility::default(),
                EditorSceneEntity,
            ))
            .id();

        Self {
            root_entity: Some(root_entity),
            selected_entity: None,
            is_modified: false,
        }
    }

    /// Select an entity in the scene tree
    pub fn select_entity(&mut self, entity: Entity) {
        self.selected_entity = Some(entity);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_entity = None;
    }

    /// Check if an entity is selected
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected_entity == Some(entity)
    }

    /// Mark scene as modified
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Mark scene as saved
    pub fn mark_saved(&mut self) {
        self.is_modified = false;
    }
}

/// Tracks whether the active project should auto-load a previously open scene.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct SceneAutoLoader {
    pub should_auto_load: bool,
    pub has_loaded: bool,
}

/// Represents a single open scene/level managed by the editor.
#[derive(Clone)]
pub struct OpenScene {
    pub name: String,
    pub file_path: Option<String>,
    pub level_data: LevelData,
    pub is_modified: bool,
    pub runtime_scene: Option<Handle<DynamicScene>>,
}

impl OpenScene {
    pub fn new(name: String, level_data: LevelData) -> Self {
        Self {
            name,
            file_path: None,
            level_data,
            is_modified: false,
            runtime_scene: None,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path_ref = path.as_ref();
        let scene = BevyScene::load_from_file(path_ref)?;
        let name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Ok(Self {
            name,
            file_path: Some(path_ref.to_string_lossy().to_string()),
            level_data: scene.data,
            is_modified: false,
            runtime_scene: None,
        })
    }
}

/// Resource managing a list of open scenes and their active tab.
#[derive(Resource)]
pub struct OpenScenes {
    pub scenes: Vec<OpenScene>,
    pub active_index: usize,
}

impl Default for OpenScenes {
    fn default() -> Self {
        Self {
            scenes: vec![OpenScene::new(
                "Untitled".to_string(),
                LevelData::new("Untitled Level".to_string(), 2000.0, 1000.0),
            )],
            active_index: 0,
        }
    }
}

impl OpenScenes {
    pub fn active_scene(&self) -> Option<&OpenScene> {
        self.scenes.get(self.active_index)
    }

    pub fn active_scene_mut(&mut self) -> Option<&mut OpenScene> {
        self.scenes.get_mut(self.active_index)
    }

    pub fn add_scene(&mut self, scene: OpenScene) {
        self.scenes.push(scene);
        self.active_index = self.scenes.len() - 1;
    }

    /// Insert a scene produced by loading from disk, replacing the default untitled
    /// scene when appropriate or appending otherwise.
    pub fn insert_loaded_scene(&mut self, scene: OpenScene) {
        if self.scenes.len() == 1
            && self.scenes[0].name.starts_with("Untitled")
            && !self.scenes[0].is_modified
        {
            self.scenes[0] = scene;
            self.active_index = 0;
        } else {
            self.add_scene(scene);
        }
    }

    pub fn close_scene(&mut self, index: usize) {
        if self.scenes.len() <= 1 {
            if let Some(scene) = self.scenes.get_mut(0) {
                *scene = OpenScene::new(
                    "Untitled".to_string(),
                    LevelData::new("Untitled Level".to_string(), 2000.0, 1000.0),
                );
            }
            self.active_index = 0;
            return;
        }

        self.scenes.remove(index);

        if self.active_index >= self.scenes.len() {
            self.active_index = self.scenes.len() - 1;
        } else if self.active_index > index {
            self.active_index -= 1;
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.scenes.len() {
            self.active_index = index;
        }
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.scenes.iter().any(|scene| scene.is_modified)
    }

    /// Get the name of the active scene (used for running scenes in the game)
    pub fn get_active_scene_name(&self) -> Option<String> {
        self.active_scene().map(|scene| {
            // Extract just the filename without extension
            scene.file_path
                .as_ref()
                .and_then(|path| {
                    std::path::Path::new(path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| scene.name.clone())
        })
    }
}

/// Load a scene from disk and merge it into the open scene collection.
pub fn load_scene_into_open_scenes<P: AsRef<Path>>(
    open_scenes: &mut OpenScenes,
    scene_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let scene = OpenScene::from_file(scene_path)?;
    open_scenes.insert_loaded_scene(scene);
    Ok(())
}

/// Message triggered when a scene tab changes.
#[derive(Event, Message)]
pub struct SceneTabChanged {
    pub new_index: usize,
}

/// Ordering buckets for scene tab change handling.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SceneTabSystemSet {
    /// Runs before the tab change is applied. Use this to persist outgoing scene state.
    Cache,
    /// Runs after caching to replace the active scene with the newly selected one.
    Apply,
}

/// Marker component for a dynamically loading scene root entity.
#[derive(Component)]
pub struct LoadingSceneRoot;

/// Prepare the world for displaying the scene at `new_index` in [`OpenScenes`].
pub fn sync_active_scene(
    commands: &mut Commands,
    editor_scene: &mut EditorScene,
    open_scenes: &OpenScenes,
    asset_server: &AssetServer,
    new_index: usize,
    existing_entities: Vec<(Entity, Option<Entity>)>,
) {
    editor_scene.selected_entity = None;

    let existing_set: std::collections::HashSet<Entity> = existing_entities
        .iter()
        .map(|(entity, _)| *entity)
        .collect();

    let mut roots = Vec::new();
    for (entity, parent) in existing_entities {
        if parent.map_or(true, |p| !existing_set.contains(&p)) {
            roots.push(entity);
        }
    }

    for entity in roots {
        commands.entity(entity).despawn();
    }

    if let Some(scene) = open_scenes.scenes.get(new_index) {
        editor_scene.is_modified = scene.is_modified;
        if let Some(file_path) = &scene.file_path {
            let scene_handle = asset_server.load::<DynamicScene>(file_path.to_string());
            let root = commands
                .spawn((
                    DynamicSceneRoot(scene_handle),
                    EditorSceneEntity,
                    LoadingSceneRoot,
                ))
                .id();
            editor_scene.root_entity = Some(root);
        } else if let Some(runtime_scene) = &scene.runtime_scene {
            let root = commands
                .spawn((
                    DynamicSceneRoot(runtime_scene.clone()),
                    EditorSceneEntity,
                    LoadingSceneRoot,
                ))
                .id();
            editor_scene.root_entity = Some(root);
        } else {
            let root = commands
                .spawn((
                    Name::new("Scene Root"),
                    Transform::default(),
                    Visibility::default(),
                    EditorSceneEntity,
                ))
                .id();
            editor_scene.root_entity = Some(root);
        }
    }
}

/// Buffer-clearing abstraction so UI crates can provide their own state.
pub trait SceneNameBuffer {
    fn clear(&mut self);
}

/// Apply the results of a tab change to the world and auxiliary buffers.
pub fn apply_scene_tab_change<B>(
    commands: &mut Commands,
    editor_scene: &mut EditorScene,
    open_scenes: &OpenScenes,
    asset_server: &AssetServer,
    new_index: usize,
    existing_entities: Vec<(Entity, Option<Entity>)>,
    name_buffer: &mut B,
) where
    B: SceneNameBuffer,
{
    sync_active_scene(
        commands,
        editor_scene,
        open_scenes,
        asset_server,
        new_index,
        existing_entities,
    );

    name_buffer.clear();
}

/// System to mark loaded scene entities with [EditorSceneEntity].
pub fn mark_loaded_scene_entities(
    mut commands: Commands,
    loading_roots: Query<Entity, With<LoadingSceneRoot>>,
    scene_instance_query: Query<&bevy::scene::SceneInstance>,
    scenes: Res<bevy::scene::SceneSpawner>,
    unmarked_entities: Query<
        (Entity, Option<&Name>, Option<&Children>, Option<&ChildOf>),
        Without<Window>,
    >,
    mut editor_scene: ResMut<EditorScene>,
) {
    for root_entity in loading_roots.iter() {
        if let Ok(scene_instance) = scene_instance_query.get(root_entity) {
            if scenes.instance_is_ready(**scene_instance) {
                info!(
                    "Scene instance ready for entity {:?}, marking spawned entities",
                    root_entity
                );

                let mut actual_root = None;
                let mut fallback_root = None;
                let mut spawned_entities = Vec::new();

                for spawned_entity in scenes.iter_instance_entities(**scene_instance) {
                    if let Ok((entity, name, children, parent)) =
                        unmarked_entities.get(spawned_entity)
                    {
                        spawned_entities.push(entity);

                        if let Some(entity_name) = name {
                            if entity_name.as_str() == "Scene Root"
                                && children.is_some_and(|c| !c.is_empty())
                            {
                                actual_root = Some(entity);
                                info!("Found actual scene root: {:?}", entity);
                            }
                        }

                        if actual_root.is_none() && parent.is_none() {
                            fallback_root = Some(entity);
                        }
                    }
                }

                for entity in &spawned_entities {
                    commands.entity(*entity).insert(EditorSceneEntity);
                }
                info!(
                    "Marked {} loaded entities as EditorSceneEntity",
                    spawned_entities.len()
                );

                let resolved_root = actual_root.or(fallback_root);

                if let Some(new_root) = resolved_root {
                    info!(
                        "Updating EditorScene.root_entity from {:?} to {:?}",
                        editor_scene.root_entity, new_root
                    );
                    editor_scene.root_entity = Some(new_root);
                } else {
                    warn!("Could not determine scene root entity after loading dynamic scene");
                }

                commands
                    .entity(root_entity)
                    .remove::<EditorSceneEntity>()
                    .remove::<LoadingSceneRoot>();
            }
        }
    }
}

/// Save current EditorScene entities to .scn.ron file
pub fn save_editor_scene_to_file(
    world: &mut World,
    scene_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get type registry for serialization
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    // Collect all EditorSceneEntity entities
    let mut scene_entities = Vec::new();
    let mut query = world.query_filtered::<Entity, With<EditorSceneEntity>>();
    for entity in query.iter(world) {
        scene_entities.push(entity);
    }

    if scene_entities.is_empty() {
        warn!("No entities to save in scene");
        return Ok(());
    }

    // Remove VisibilityClass from all entities before serialization
    // (contains TypeId which can't be serialized)
    for &entity in &scene_entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.remove::<bevy::camera::visibility::VisibilityClass>();
        }
    }

    // Build DynamicScene from all scene entities
    let scene_builder =
        DynamicSceneBuilder::from_world(world).extract_entities(scene_entities.into_iter());
    let dynamic_scene = scene_builder.build();

    // Serialize to RON
    let type_registry = type_registry.read();
    let ron_string = dynamic_scene.serialize(&type_registry)?;

    // Write to file
    std::fs::write(scene_path, ron_string)?;
    info!("Scene saved to: {}", scene_path);

    Ok(())
}

/// Capture the current editor scene into an in-memory [`DynamicScene`].
pub fn capture_editor_scene_runtime(world: &mut World) -> DynamicScene {
    let mut scene_entities = Vec::new();
    let mut query = world.query_filtered::<Entity, With<EditorSceneEntity>>();
    for entity in query.iter(world) {
        scene_entities.push(entity);
    }

    DynamicSceneBuilder::from_world(world)
        .deny_component::<bevy::camera::visibility::VisibilityClass>()
        .extract_entities(scene_entities.into_iter())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::Assets;
    use bevy::ecs::entity::EntityHashMap;
    use bevy::prelude::{App, Image, MinimalPlugins, Name, Transform};

    #[test]
    fn captured_runtime_scene_includes_name_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SceneEditorPlugin);
        app.world_mut().insert_resource(Assets::<Image>::default());
        app.update();

        let entity_id = {
            app.world_mut()
                .spawn((
                    Name::new("Test Entity"),
                    Transform::default(),
                    EditorSceneEntity,
                ))
                .id()
        };

        let dynamic_scene = {
            let mut world = app.world_mut();
            capture_editor_scene_runtime(&mut world)
        };

        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let serialization = dynamic_scene
            .serialize(&registry.read())
            .expect("serialize dynamic scene");

        assert!(
            serialization.contains("Test Entity"),
            "Serialized scene should contain entity name; got: {serialization}"
        );

        // Remove the original entity, then re-apply the scene into the world.
        {
            let mut world = app.world_mut();
            world.despawn(entity_id);
            dynamic_scene
                .write_to_world(&mut world, &mut EntityHashMap::default())
                .expect("write scene back to world");
        }

        // Verify that at least one entity in the world carries the original name.
        let found = app
            .world()
            .iter_entities()
            .filter_map(|entity| entity.get::<Name>())
            .any(|name| name.as_str() == "Test Entity");
        assert!(found, "Deserialized scene should restore entity name");
    }
}

/// Load .scn.ron file into EditorScene using Bevy's asset system
pub fn load_editor_scene_from_file(
    commands: &mut Commands,
    asset_server: &AssetServer,
    scene_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading scene from: {}", scene_path);

    // Load scene using Bevy's asset system
    let scene_handle: Handle<DynamicScene> = asset_server.load(scene_path.to_string());

    // Spawn scene into world with marker
    commands.spawn((
        DynamicSceneRoot(scene_handle.clone()),
        EditorSceneEntity, // Mark the root so we can find it
    ));

    Ok(())
}

/// System to tag newly spawned scene entities with EditorSceneEntity marker
/// This runs after SceneSpawner has instantiated the scene
pub fn tag_spawned_scene_entities(
    mut commands: Commands,
    untagged_query: Query<Entity, (Without<EditorSceneEntity>, With<Transform>)>,
    scene_root_query: Query<&DynamicSceneRoot, With<EditorSceneEntity>>,
    mut editor_scene: ResMut<EditorScene>,
) {
    // If we have a scene root marker, tag all untagged Transform entities
    if scene_root_query.iter().count() > 0 {
        let mut tagged_count = 0;
        for entity in untagged_query.iter() {
            commands.entity(entity).insert(EditorSceneEntity);
            tagged_count += 1;

            // Set first tagged entity as root if not set
            if editor_scene.root_entity.is_none() {
                editor_scene.root_entity = Some(entity);
                info!("Set scene root to {:?}", entity);
            }
        }

        if tagged_count > 0 {
            info!("Tagged {} entities as EditorSceneEntity", tagged_count);
        }
    }
}

/// System to initialize editor scene on startup
pub fn setup_editor_scene(mut commands: Commands) {
    let editor_scene = EditorScene::new(&mut commands);
    commands.insert_resource(editor_scene);
}

/// Message for editing entity transforms
#[derive(Event, Message, Debug, Clone)]
pub enum TransformEditEvent {
    /// Set position (replaces current position)
    SetPosition { entity: Entity, position: Vec2 },
    /// Translate by delta (adds to current position)
    Translate { entity: Entity, delta: Vec2 },
    /// Set rotation (replaces current rotation)
    SetRotation { entity: Entity, rotation: f32 },
    /// Set scale (replaces current scale)
    SetScale { entity: Entity, scale: Vec2 },
}

/// Message for editing entity name
#[derive(Event, Message, Debug, Clone)]
pub struct NameEditEvent {
    pub entity: Entity,
    pub new_name: String,
}

/// Message for assigning texture to sprite
#[derive(Event, Message, Debug, Clone)]
pub struct SpriteTextureEvent {
    pub entity: Entity,
    pub texture_handle: Handle<Image>,
}

/// Message for adding a component to an entity
#[derive(Event, Message, Debug, Clone)]
pub struct AddComponentEvent {
    pub entity: Entity,
    pub component_name: String,
}

/// Message for removing a component from an entity
#[derive(Event, Message, Debug, Clone)]
pub struct RemoveComponentEvent {
    pub entity: Entity,
    pub component_name: String,
}

/// System to handle transform edit messages
pub fn handle_transform_edit_events(
    mut events: MessageReader<TransformEditEvent>,
    mut entity_query: Query<&mut Transform, With<EditorSceneEntity>>,
    mut editor_scene: ResMut<EditorScene>,
) {
    for event in events.read() {
        match event {
            TransformEditEvent::SetPosition { entity, position } => {
                if let Ok(mut transform) = entity_query.get_mut(*entity) {
                    transform.translation.x = position.x;
                    transform.translation.y = position.y;
                    editor_scene.mark_modified();
                    info!("Set entity {:?} position to {:?}", entity, position);
                }
            }
            TransformEditEvent::Translate { entity, delta } => {
                if let Ok(mut transform) = entity_query.get_mut(*entity) {
                    transform.translation.x += delta.x;
                    transform.translation.y += delta.y;
                    editor_scene.mark_modified();
                    info!("Translated entity {:?} by {:?}", entity, delta);
                }
            }
            TransformEditEvent::SetRotation { entity, rotation } => {
                if let Ok(mut transform) = entity_query.get_mut(*entity) {
                    transform.rotation = Quat::from_rotation_z(*rotation);
                    editor_scene.mark_modified();
                    info!("Set entity {:?} rotation to {}", entity, rotation);
                }
            }
            TransformEditEvent::SetScale { entity, scale } => {
                if let Ok(mut transform) = entity_query.get_mut(*entity) {
                    transform.scale.x = scale.x;
                    transform.scale.y = scale.y;
                    editor_scene.mark_modified();
                    info!("Set entity {:?} scale to {:?}", entity, scale);
                }
            }
        }
    }
}

/// System to handle name edit messages
pub fn handle_name_edit_events(
    mut events: MessageReader<NameEditEvent>,
    mut entity_query: Query<&mut Name, With<EditorSceneEntity>>,
    mut editor_scene: ResMut<EditorScene>,
) {
    for event in events.read() {
        if let Ok(mut name) = entity_query.get_mut(event.entity) {
            name.set(event.new_name.clone());
            editor_scene.mark_modified();
            info!("Renamed entity {:?} to '{}'", event.entity, event.new_name);
        }
    }
}

/// System to handle sprite texture assignment messages
pub fn handle_sprite_texture_events(
    mut events: MessageReader<SpriteTextureEvent>,
    mut sprite_query: Query<&mut Sprite, With<EditorSceneEntity>>,
    mut editor_scene: ResMut<EditorScene>,
    images: Res<Assets<Image>>,
) {
    for event in events.read() {
        // Update sprite's image handle and reset color to white for proper texture display
        if let Ok(mut sprite) = sprite_query.get_mut(event.entity) {
            sprite.image = event.texture_handle.clone();
            // Set color to white so the texture displays without tinting
            sprite.color = Color::WHITE;

            // Update custom_size to match texture dimensions if texture is loaded
            if let Some(image) = images.get(&event.texture_handle) {
                sprite.custom_size = Some(image.size().as_vec2());
                info!(
                    "Set sprite custom_size to texture dimensions: {:?}",
                    image.size()
                );
            } else {
                // Texture not loaded yet - remove custom_size to use natural texture size when it loads
                sprite.custom_size = None;
            }

            editor_scene.mark_modified();
            info!("Assigned texture to sprite entity {:?}", event.entity);
        } else {
            warn!(
                "Attempted to assign texture to non-sprite entity {:?}",
                event.entity
            );
        }
    }
}

/// Plugin for scene editor functionality
pub struct SceneEditorPlugin;

impl Plugin for SceneEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorScene>()
            // Register marker component
            .register_type::<EditorSceneEntity>()
            // Register core Bevy components for scene serialization
            .register_type::<Name>()
            .register_type::<Transform>()
            .register_type::<GlobalTransform>()
            .register_type::<Visibility>()
            .register_type::<InheritedVisibility>()
            .register_type::<ViewVisibility>()
            // Register rendering components
            .register_type::<Sprite>()
            // Events
            .add_message::<TransformEditEvent>()
            .add_message::<NameEditEvent>()
            .add_message::<SpriteTextureEvent>()
            .add_message::<AddComponentEvent>()
            .add_message::<RemoveComponentEvent>()
            // Systems
            .add_systems(Startup, setup_editor_scene)
            .add_systems(
                Update,
                (
                    handle_transform_edit_events,
                    handle_name_edit_events,
                    handle_sprite_texture_events,
                    tag_spawned_scene_entities, // Tag entities after scene loads
                ),
            );
    }
}
