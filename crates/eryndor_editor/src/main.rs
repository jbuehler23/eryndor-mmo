//! Eryndor Game Content Editor
//! A web-based design toolkit for creating game content.

// Editor is still in development - allow unused code and stylistic warnings
#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::op_ref)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::unnecessary_cast)]

mod api_client;
mod api_events;
mod editor_state;
mod modules;
mod ui;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

use api_events::ApiEventsPlugin;
use editor_state::{EditorState, EditorTab, TilesetDefinition, TileSource, TileCategory};
use ui::{render_main_menu, render_tab_bar, render_status_bar, render_error_popup};

fn main() {
    // Set up panic hook for WASM
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Eryndor Editor".to_string(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(ApiEventsPlugin)
        .init_resource::<EditorState>()
        .add_systems(Startup, setup)
        // UI systems must be in EguiPrimaryContextPass for bevy_egui 0.38
        .add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system)
        .add_systems(Update, (
            process_zone_item_actions,
            process_enemy_actions,
        ))
        .add_systems(Update, (
            process_npc_quest_actions,
            process_ability_loot_actions,
        ))
        .add_systems(Update, (
            load_tile_textures,
            register_tile_textures,
            load_tileset_textures,
            register_tileset_textures,
            update_tileset_dimensions,
        ))
        .run();
}

fn setup(mut commands: Commands, mut editor_state: ResMut<EditorState>) {
    // Spawn 2D camera for the editor
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.12)),
            ..default()
        },
    ));

    // Initialize tile palette with static data from tile_palette.tiles.json structure
    initialize_tile_palette(&mut editor_state);

    // Initialize new tileset system
    initialize_tilesets(&mut editor_state);

    info!("Eryndor Editor initialized");
}

/// Initialize the tile palette with known tiles from the asset structure
fn initialize_tile_palette(editor_state: &mut EditorState) {
    use editor_state::TilePaletteEntry;

    // Ground tiles
    editor_state.world.tile_palette.ground_tiles = vec![
        TilePaletteEntry { id: 1, name: "Grass 1".to_string(), path: "tiles/Tiles/Grass/Grass_1_Middle.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 2, name: "Grass 2".to_string(), path: "tiles/Tiles/Grass/Grass_2_Middle.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 3, name: "Grass 3".to_string(), path: "tiles/Tiles/Grass/Grass_3_Middle.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 4, name: "Grass 4".to_string(), path: "tiles/Tiles/Grass/Grass_4_Middle.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 5, name: "Grass Tiles 2".to_string(), path: "tiles/Tiles/Grass/Grass_Tiles_2.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 6, name: "Grass Tiles 3".to_string(), path: "tiles/Tiles/Grass/Grass_Tiles_3.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 7, name: "Grass Tiles 4".to_string(), path: "tiles/Tiles/Grass/Grass_Tiles_4.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 10, name: "Path Middle".to_string(), path: "tiles/Tiles/Grass/Path_Middle.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 11, name: "Path Decor".to_string(), path: "tiles/Tiles/Grass/Path_Decoration.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 20, name: "Cobble Road 1".to_string(), path: "tiles/Tiles/Cobble_Road/Cobble_Road_1.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 21, name: "Cobble Road 2".to_string(), path: "tiles/Tiles/Cobble_Road/Cobble_Road_2.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 30, name: "Pavement".to_string(), path: "tiles/Tiles/Pavement_Tiles.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 40, name: "Water".to_string(), path: "tiles/Tiles/Water/Water_Middle.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 41, name: "Water Tile 1".to_string(), path: "tiles/Tiles/Water/Water_Tile_1.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 42, name: "Water Tile 2".to_string(), path: "tiles/Tiles/Water/Water_Tile_2.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 43, name: "Water Tile 3".to_string(), path: "tiles/Tiles/Water/Water_Tile_3.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 44, name: "Water Tile 4".to_string(), path: "tiles/Tiles/Water/Water_Tile_4.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 50, name: "Beach".to_string(), path: "tiles/Tiles/Beach/Beach_Tiles.png".to_string(), has_collision: false },
    ];

    // Decoration tiles
    editor_state.world.tile_palette.decoration_tiles = vec![
        TilePaletteEntry { id: 100, name: "Big Oak".to_string(), path: "tiles/Trees/Big_Oak_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 101, name: "Big Birch".to_string(), path: "tiles/Trees/Big_Birch_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 102, name: "Big Spruce".to_string(), path: "tiles/Trees/Big_Spruce_tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 103, name: "Big Fruit".to_string(), path: "tiles/Trees/Big_Fruit_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 110, name: "Med Oak".to_string(), path: "tiles/Trees/Medium_Oak_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 111, name: "Med Birch".to_string(), path: "tiles/Trees/Medium_Birch_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 112, name: "Med Spruce".to_string(), path: "tiles/Trees/Medium_Spruce_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 113, name: "Med Fruit".to_string(), path: "tiles/Trees/Medium_Fruit_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 120, name: "Small Oak".to_string(), path: "tiles/Trees/Small_Oak_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 121, name: "Small Birch".to_string(), path: "tiles/Trees/Small_Birch_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 122, name: "Small Spruce".to_string(), path: "tiles/Trees/Small_Spruce_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 123, name: "Small Fruit".to_string(), path: "tiles/Trees/Small_Fruit_Tree.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 150, name: "Flowers".to_string(), path: "tiles/Outdoor decoration/Flowers.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 151, name: "Fountain".to_string(), path: "tiles/Outdoor decoration/Fountain.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 152, name: "Well".to_string(), path: "tiles/Outdoor decoration/Well.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 153, name: "Benches".to_string(), path: "tiles/Outdoor decoration/Benches.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 154, name: "Fences".to_string(), path: "tiles/Outdoor decoration/Fences.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 155, name: "Fence Big".to_string(), path: "tiles/Outdoor decoration/Fence_Big.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 156, name: "Barrels".to_string(), path: "tiles/Outdoor decoration/barrels.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 157, name: "Hay Bales".to_string(), path: "tiles/Outdoor decoration/Hay_Bales.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 158, name: "Lanterns".to_string(), path: "tiles/Outdoor decoration/Lanter_Posts.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 159, name: "Camp Decor".to_string(), path: "tiles/Outdoor decoration/Camp_Decor.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 160, name: "Outdoor Decor".to_string(), path: "tiles/Outdoor decoration/Outdoor_Decor.png".to_string(), has_collision: false },
        TilePaletteEntry { id: 161, name: "Signs".to_string(), path: "tiles/Outdoor decoration/Signs.png".to_string(), has_collision: true },
        TilePaletteEntry { id: 162, name: "Boat".to_string(), path: "tiles/Outdoor decoration/Boat.png".to_string(), has_collision: true },
    ];

    editor_state.world.tile_palette.loaded = true;

    // Mark that we need to load textures
    editor_state.world.tile_palette.textures_loading = true;
    info!("Tile palette initialized with {} ground tiles and {} decoration tiles",
        editor_state.world.tile_palette.ground_tiles.len(),
        editor_state.world.tile_palette.decoration_tiles.len());
}

fn editor_ui_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
) {
    // Get the egui context - returns Result in bevy_egui 0.38
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Main menu bar at top
    render_main_menu(ctx, &mut editor_state);

    // Tab bar below menu
    render_tab_bar(ctx, &mut editor_state);

    // Status bar at very bottom (render FIRST to claim outermost bottom position)
    render_status_bar(ctx, &editor_state);

    // Render World tab's bottom palette panel at context level (before CentralPanel)
    // This renders AFTER status bar so it appears ABOVE the status bar
    if editor_state.active_tab == EditorTab::World {
        modules::world::render_bottom_panel(ctx, &mut editor_state);
    }

    // Main content area based on active tab
    egui::CentralPanel::default().show(ctx, |ui| {
        match editor_state.active_tab {
            EditorTab::World => modules::world::render(ui, &mut editor_state),
            EditorTab::Tilesets => modules::tilesets::render(ui, &mut editor_state),
            EditorTab::Items => modules::items::render(ui, &mut editor_state),
            EditorTab::Enemies => modules::enemies::render(ui, &mut editor_state),
            EditorTab::Npcs => modules::npcs::render(ui, &mut editor_state),
            EditorTab::Quests => modules::quests::render(ui, &mut editor_state),
            EditorTab::Abilities => modules::abilities::render(ui, &mut editor_state),
            EditorTab::Loot => modules::loot::render(ui, &mut editor_state),
            EditorTab::Assets => modules::assets::render(ui, &mut editor_state),
        }
    });

    // Error popup (renders on top of everything when there's an error)
    render_error_popup(ctx, &mut editor_state);
}

/// System to process zone and item actions
fn process_zone_item_actions(
    mut editor_state: ResMut<EditorState>,
    mut load_zone_events: MessageWriter<api_events::LoadZoneListEvent>,
    mut create_zone_events: MessageWriter<api_events::CreateZoneEvent>,
    mut load_item_events: MessageWriter<api_events::LoadItemListEvent>,
    mut create_item_events: MessageWriter<api_events::CreateItemEvent>,
    mut update_item_events: MessageWriter<api_events::UpdateItemEvent>,
    mut delete_item_events: MessageWriter<api_events::DeleteItemEvent>,
    mut save_tilemap_events: MessageWriter<api_events::SaveTilemapEvent>,
) {
    // Process load zones action
    if editor_state.action_load_zones {
        editor_state.action_load_zones = false;
        editor_state.status_message = "Loading zones...".to_string();
        load_zone_events.write(api_events::LoadZoneListEvent);
    }

    // Process create zone action
    if editor_state.action_create_zone {
        editor_state.action_create_zone = false;

        let zone_name = editor_state.world.new_zone_name.trim().to_string();
        if zone_name.is_empty() {
            editor_state.status_message = "Zone name cannot be empty".to_string();
            return;
        }

        let zone_id = zone_name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        let zone_data = api_events::ZoneData {
            id: zone_id,
            name: zone_name,
            width: editor_state.world.new_zone_width,
            height: editor_state.world.new_zone_height,
            ..Default::default()
        };

        editor_state.status_message = format!("Creating zone: {}...", zone_data.name);
        create_zone_events.write(api_events::CreateZoneEvent { zone: zone_data });

        editor_state.world.new_zone_name.clear();
        editor_state.world.show_create_dialog = false;
    }

    // Process load items action
    if editor_state.action_load_items {
        editor_state.action_load_items = false;
        editor_state.status_message = "Loading items...".to_string();
        load_item_events.write(api_events::LoadItemListEvent);
    }

    // Process create item action
    if editor_state.action_create_item {
        editor_state.action_create_item = false;

        let item_name = editor_state.items.new_item_name.trim().to_string();
        if item_name.is_empty() {
            editor_state.status_message = "Item name cannot be empty".to_string();
            return;
        }

        let item_id = (editor_state.items.item_list.len() as u32 + 1000) as u32;

        let item_type = if editor_state.items.new_item_type.is_empty() {
            "Weapon".to_string()
        } else {
            editor_state.items.new_item_type.clone()
        };

        let item_data = api_events::ItemData {
            id: item_id,
            name: item_name,
            item_type,
            grants_ability: None,
            stat_bonuses: api_events::ItemStatBonuses::default(),
        };

        editor_state.status_message = format!("Creating item: {}...", item_data.name);
        create_item_events.write(api_events::CreateItemEvent { item: item_data });

        editor_state.items.new_item_name.clear();
        editor_state.items.new_item_type.clear();
        editor_state.items.show_create_dialog = false;
    }

    // Process save item action
    if editor_state.action_save_item {
        editor_state.action_save_item = false;

        if let Some(ref editing_item) = editor_state.items.editing_item {
            let item_data = api_events::ItemData {
                id: editing_item.id,
                name: editing_item.name.clone(),
                item_type: editing_item.item_type.clone(),
                grants_ability: editing_item.grants_ability,
                stat_bonuses: api_events::ItemStatBonuses {
                    attack_power: editing_item.attack_power,
                    defense: editing_item.defense,
                    max_health: editing_item.max_health,
                    max_mana: editing_item.max_mana,
                    crit_chance: editing_item.crit_chance,
                },
            };

            editor_state.status_message = format!("Saving item: {}...", item_data.name);
            update_item_events.write(api_events::UpdateItemEvent { item: item_data });
        }
    }

    // Process delete item action
    if editor_state.action_delete_item {
        editor_state.action_delete_item = false;

        if let Some(item_id) = editor_state.items.selected_item {
            editor_state.status_message = format!("Deleting item {}...", item_id);
            delete_item_events.write(api_events::DeleteItemEvent { item_id });
        }
    }

    // Process save tilemap action
    if editor_state.action_save_tilemap {
        editor_state.action_save_tilemap = false;

        if let (Some(zone_id), Some(tilemap)) = (
            editor_state.world.current_zone.clone(),
            editor_state.world.editing_tilemap.clone(),
        ) {
            editor_state.status_message = format!("Saving tilemap for zone {}...", zone_id);
            save_tilemap_events.write(api_events::SaveTilemapEvent { zone_id, tilemap });
        } else {
            editor_state.status_message = "Cannot save: No zone selected or no tilemap".to_string();
        }
    }
}

/// System to process enemy actions
fn process_enemy_actions(
    mut editor_state: ResMut<EditorState>,
    mut load_enemy_events: MessageWriter<api_events::LoadEnemyListEvent>,
    mut create_enemy_events: MessageWriter<api_events::CreateEnemyEvent>,
    mut update_enemy_events: MessageWriter<api_events::UpdateEnemyEvent>,
    mut delete_enemy_events: MessageWriter<api_events::DeleteEnemyEvent>,
) {
    if editor_state.action_load_enemies {
        editor_state.action_load_enemies = false;
        editor_state.status_message = "Loading enemies...".to_string();
        load_enemy_events.write(api_events::LoadEnemyListEvent);
    }

    if editor_state.action_create_enemy {
        editor_state.action_create_enemy = false;

        let enemy_name = editor_state.enemies.new_enemy_name.trim().to_string();
        if enemy_name.is_empty() {
            editor_state.status_message = "Enemy name cannot be empty".to_string();
            return;
        }

        let enemy_id = (editor_state.enemies.enemy_list.len() as u32 + 1000) as u32;

        let enemy_data = api_events::EnemyData {
            id: enemy_id,
            name: enemy_name,
            max_health: 100.0,
            attack_power: 10.0,
            defense: 5.0,
            move_speed: 100.0,
            aggro_range: 150.0,
            leash_range: 300.0,
            respawn_delay: 10.0,
            loot_table: api_events::EnemyLootTable {
                gold_min: 5,
                gold_max: 15,
                items: Vec::new(),
            },
            visual: api_events::EnemyVisual {
                shape: "Circle".to_string(),
                color: [0.8, 0.2, 0.2, 1.0],
                size: 16.0,
            },
        };

        editor_state.status_message = format!("Creating enemy: {}...", enemy_data.name);
        create_enemy_events.write(api_events::CreateEnemyEvent { enemy: enemy_data });

        editor_state.enemies.new_enemy_name.clear();
        editor_state.enemies.show_create_dialog = false;
    }

    if editor_state.action_save_enemy {
        editor_state.action_save_enemy = false;

        if let Some(ref editing_enemy) = editor_state.enemies.editing_enemy {
            // Convert loot items from editing format to API format
            let loot_items: Vec<api_events::EnemyLootItem> = editing_enemy
                .loot_items
                .iter()
                .map(|item| api_events::EnemyLootItem {
                    item_id: item.item_id,
                    drop_chance: item.drop_chance,
                    quantity_min: item.quantity_min,
                    quantity_max: item.quantity_max,
                })
                .collect();

            let enemy_data = api_events::EnemyData {
                id: editing_enemy.id,
                name: editing_enemy.name.clone(),
                max_health: editing_enemy.max_health,
                attack_power: editing_enemy.attack_power,
                defense: editing_enemy.defense,
                move_speed: editing_enemy.move_speed,
                aggro_range: editing_enemy.aggro_range,
                leash_range: editing_enemy.leash_range,
                respawn_delay: editing_enemy.respawn_delay,
                loot_table: api_events::EnemyLootTable {
                    gold_min: editing_enemy.gold_min,
                    gold_max: editing_enemy.gold_max,
                    items: loot_items,
                },
                visual: api_events::EnemyVisual {
                    shape: editing_enemy.visual_shape.clone(),
                    color: editing_enemy.visual_color,
                    size: editing_enemy.visual_size,
                },
            };

            editor_state.status_message = format!("Saving enemy: {}...", enemy_data.name);
            update_enemy_events.write(api_events::UpdateEnemyEvent { enemy: enemy_data });
        }
    }

    if editor_state.action_delete_enemy {
        editor_state.action_delete_enemy = false;

        if let Some(enemy_id) = editor_state.enemies.selected_enemy {
            editor_state.status_message = format!("Deleting enemy {}...", enemy_id);
            delete_enemy_events.write(api_events::DeleteEnemyEvent { enemy_id });
        }
    }
}

/// System to process NPC and quest actions
fn process_npc_quest_actions(
    mut editor_state: ResMut<EditorState>,
    mut load_npc_events: MessageWriter<api_events::LoadNpcListEvent>,
    mut create_npc_events: MessageWriter<api_events::CreateNpcEvent>,
    mut update_npc_events: MessageWriter<api_events::UpdateNpcEvent>,
    mut delete_npc_events: MessageWriter<api_events::DeleteNpcEvent>,
    mut load_quest_events: MessageWriter<api_events::LoadQuestListEvent>,
    mut create_quest_events: MessageWriter<api_events::CreateQuestEvent>,
    mut update_quest_events: MessageWriter<api_events::UpdateQuestEvent>,
    mut delete_quest_events: MessageWriter<api_events::DeleteQuestEvent>,
) {
    // NPC Actions
    if editor_state.action_load_npcs {
        editor_state.action_load_npcs = false;
        editor_state.status_message = "Loading NPCs...".to_string();
        load_npc_events.write(api_events::LoadNpcListEvent);
    }

    if editor_state.action_create_npc {
        editor_state.action_create_npc = false;

        let npc_name = editor_state.npcs.new_npc_name.trim().to_string();
        if npc_name.is_empty() {
            editor_state.status_message = "NPC name cannot be empty".to_string();
            return;
        }

        let npc_id = (editor_state.npcs.npc_list.len() as u32 + 1000) as u32;

        let npc_type = if editor_state.npcs.new_npc_role.is_empty() {
            "QuestGiver".to_string()
        } else {
            editor_state.npcs.new_npc_role.clone()
        };

        let npc_data = api_events::NpcData {
            id: npc_id,
            name: npc_name,
            npc_type,
            position: api_events::NpcPosition { x: 0.0, y: 0.0 },
            quests: Vec::new(),
            trainer_items: Vec::new(),
            visual: api_events::VisualData::default(),
        };

        editor_state.status_message = format!("Creating NPC: {}...", npc_data.name);
        create_npc_events.write(api_events::CreateNpcEvent { npc: npc_data });

        editor_state.npcs.new_npc_name.clear();
        editor_state.npcs.new_npc_role.clear();
        editor_state.npcs.show_create_dialog = false;
    }

    if editor_state.action_save_npc {
        editor_state.action_save_npc = false;

        if let Some(ref editing_npc) = editor_state.npcs.editing_npc {
            let npc_data = api_events::NpcData {
                id: editing_npc.id,
                name: editing_npc.name.clone(),
                npc_type: editing_npc.npc_type.clone(),
                position: api_events::NpcPosition {
                    x: editing_npc.position_x,
                    y: editing_npc.position_y,
                },
                quests: editing_npc.quests.clone(),
                trainer_items: editing_npc.trainer_items.iter().map(|item| {
                    api_events::TrainerItemData {
                        item_id: item.item_id,
                        cost: item.cost,
                    }
                }).collect(),
                visual: api_events::VisualData {
                    shape: editing_npc.visual_shape.clone(),
                    color: editing_npc.visual_color,
                    size: editing_npc.visual_size,
                },
            };

            editor_state.status_message = format!("Saving NPC: {}...", npc_data.name);
            update_npc_events.write(api_events::UpdateNpcEvent { npc: npc_data });
        }
    }

    if editor_state.action_delete_npc {
        editor_state.action_delete_npc = false;

        if let Some(npc_id) = editor_state.npcs.selected_npc {
            editor_state.status_message = format!("Deleting NPC {}...", npc_id);
            delete_npc_events.write(api_events::DeleteNpcEvent { npc_id });
        }
    }

    // Quest Actions
    if editor_state.action_load_quests {
        editor_state.action_load_quests = false;
        editor_state.status_message = "Loading quests...".to_string();
        load_quest_events.write(api_events::LoadQuestListEvent);
    }

    if editor_state.action_create_quest {
        editor_state.action_create_quest = false;

        let quest_name = editor_state.quests.new_quest_name.trim().to_string();
        if quest_name.is_empty() {
            editor_state.status_message = "Quest name cannot be empty".to_string();
            return;
        }

        let quest_id = (editor_state.quests.quest_list.len() as u32 + 1000) as u32;

        let quest_data = api_events::QuestData {
            id: quest_id,
            name: quest_name,
            description: String::new(),
            objectives: Vec::new(),
            reward_exp: 100,
            proficiency_requirements: Vec::new(),
            reward_abilities: Vec::new(),
        };

        editor_state.status_message = format!("Creating quest: {}...", quest_data.name);
        create_quest_events.write(api_events::CreateQuestEvent { quest: quest_data });

        editor_state.quests.new_quest_name.clear();
        editor_state.quests.new_quest_type.clear();
        editor_state.quests.show_create_dialog = false;
    }

    if editor_state.action_save_quest {
        editor_state.action_save_quest = false;

        if let Some(ref editing_quest) = editor_state.quests.editing_quest {
            // Convert objectives to JSON
            let objectives: Vec<serde_json::Value> = editing_quest.objectives.iter().map(|obj| {
                use editor_state::EditingQuestObjective;
                match obj {
                    EditingQuestObjective::TalkToNpc { npc_id } => {
                        serde_json::json!({"type": "TalkToNpc", "npc_id": npc_id})
                    }
                    EditingQuestObjective::KillEnemy { enemy_type, count } => {
                        serde_json::json!({"type": "KillEnemy", "enemy_type": enemy_type, "count": count})
                    }
                    EditingQuestObjective::ObtainItem { item_id, count } => {
                        serde_json::json!({"type": "ObtainItem", "item_id": item_id, "count": count})
                    }
                }
            }).collect();

            // Convert proficiency requirements to JSON
            let proficiency_requirements: Vec<serde_json::Value> = editing_quest.proficiency_requirements.iter().map(|req| {
                serde_json::json!({"weapon_type": req.weapon_type, "level": req.level})
            }).collect();

            let quest_data = api_events::QuestData {
                id: editing_quest.id,
                name: editing_quest.name.clone(),
                description: editing_quest.description.clone(),
                objectives,
                reward_exp: editing_quest.reward_exp,
                proficiency_requirements,
                reward_abilities: editing_quest.reward_abilities.iter().map(|id| format!("ability_{}", id)).collect(),
            };

            editor_state.status_message = format!("Saving quest: {}...", quest_data.name);
            update_quest_events.write(api_events::UpdateQuestEvent { quest: quest_data });
        }
    }

    if editor_state.action_delete_quest {
        editor_state.action_delete_quest = false;

        if let Some(quest_id) = editor_state.quests.selected_quest {
            editor_state.status_message = format!("Deleting quest {}...", quest_id);
            delete_quest_events.write(api_events::DeleteQuestEvent { quest_id });
        }
    }
}

/// System to process ability and loot table actions
fn process_ability_loot_actions(
    mut editor_state: ResMut<EditorState>,
    mut load_ability_events: MessageWriter<api_events::LoadAbilityListEvent>,
    mut create_ability_events: MessageWriter<api_events::CreateAbilityEvent>,
    mut update_ability_events: MessageWriter<api_events::UpdateAbilityEvent>,
    mut delete_ability_events: MessageWriter<api_events::DeleteAbilityEvent>,
    mut load_loot_table_events: MessageWriter<api_events::LoadLootTableListEvent>,
    mut create_loot_table_events: MessageWriter<api_events::CreateLootTableEvent>,
    mut update_loot_table_events: MessageWriter<api_events::UpdateLootTableEvent>,
    mut delete_loot_table_events: MessageWriter<api_events::DeleteLootTableEvent>,
) {
    // Ability Actions
    if editor_state.action_load_abilities {
        editor_state.action_load_abilities = false;
        editor_state.status_message = "Loading abilities...".to_string();
        load_ability_events.write(api_events::LoadAbilityListEvent);
    }

    if editor_state.action_create_ability {
        editor_state.action_create_ability = false;

        let ability_name = editor_state.abilities.new_ability_name.trim().to_string();
        if ability_name.is_empty() {
            editor_state.status_message = "Ability name cannot be empty".to_string();
            return;
        }

        let ability_id = (editor_state.abilities.ability_list.len() as u32 + 1000) as u32;

        let ability_data = api_events::AbilityData {
            id: ability_id,
            name: ability_name,
            description: String::new(),
            damage_multiplier: 1.0,
            cooldown: 3.0,
            range: 1.5,
            mana_cost: 20.0,
            ability_types: vec![serde_json::json!({"DirectDamage": {"multiplier": 1.0}})],
            unlock_requirement: serde_json::json!("None"),
        };

        editor_state.status_message = format!("Creating ability: {}...", ability_data.name);
        create_ability_events.write(api_events::CreateAbilityEvent { ability: ability_data });

        editor_state.abilities.new_ability_name.clear();
        editor_state.abilities.new_ability_type.clear();
        editor_state.abilities.show_create_dialog = false;
    }

    if editor_state.action_save_ability {
        editor_state.action_save_ability = false;

        if let Some(ref editing_ability) = editor_state.abilities.editing_ability {
            // Convert EditingAbility effects to JSON
            let ability_types: Vec<serde_json::Value> = editing_ability.ability_effects.iter().map(|effect| {
                use editor_state::{EditingAbilityEffect, EditingDebuffType};
                match effect {
                    EditingAbilityEffect::DirectDamage { multiplier } => {
                        serde_json::json!({"DirectDamage": {"multiplier": multiplier}})
                    }
                    EditingAbilityEffect::DamageOverTime { duration, ticks, damage_per_tick } => {
                        serde_json::json!({"DamageOverTime": {"duration": duration, "ticks": ticks, "damage_per_tick": damage_per_tick}})
                    }
                    EditingAbilityEffect::AreaOfEffect { radius, max_targets } => {
                        serde_json::json!({"AreaOfEffect": {"radius": radius, "max_targets": max_targets}})
                    }
                    EditingAbilityEffect::Buff { duration, attack_power, defense, move_speed } => {
                        serde_json::json!({"Buff": {"duration": duration, "stat_bonuses": {"attack_power": attack_power, "defense": defense, "move_speed": move_speed}}})
                    }
                    EditingAbilityEffect::Debuff { duration, debuff_type } => {
                        let effect_json = match debuff_type {
                            EditingDebuffType::Stun => serde_json::json!("Stun"),
                            EditingDebuffType::Root => serde_json::json!("Root"),
                            EditingDebuffType::Slow { move_speed_reduction } => serde_json::json!({"Slow": {"move_speed_reduction": move_speed_reduction}}),
                            EditingDebuffType::Weaken { attack_reduction } => serde_json::json!({"Weaken": {"attack_reduction": attack_reduction}}),
                        };
                        serde_json::json!({"Debuff": {"duration": duration, "effect": effect_json}})
                    }
                    EditingAbilityEffect::Mobility { distance, dash_speed } => {
                        serde_json::json!({"Mobility": {"distance": distance, "dash_speed": dash_speed}})
                    }
                    EditingAbilityEffect::Heal { amount, is_percent } => {
                        serde_json::json!({"Heal": {"amount": amount, "is_percent": is_percent}})
                    }
                }
            }).collect();

            // Convert unlock requirement to JSON
            let unlock_requirement = {
                use editor_state::EditingUnlockRequirement;
                match &editing_ability.unlock_requirement {
                    EditingUnlockRequirement::None => serde_json::json!("None"),
                    EditingUnlockRequirement::Level(lvl) => serde_json::json!({"Level": lvl}),
                    EditingUnlockRequirement::Quest(qid) => serde_json::json!({"Quest": qid}),
                    EditingUnlockRequirement::WeaponProficiency { weapon, level } => {
                        serde_json::json!({"WeaponProficiency": {"weapon": weapon, "level": level}})
                    }
                }
            };

            let ability_data = api_events::AbilityData {
                id: editing_ability.id,
                name: editing_ability.name.clone(),
                description: editing_ability.description.clone(),
                damage_multiplier: editing_ability.damage_multiplier,
                cooldown: editing_ability.cooldown,
                range: editing_ability.range,
                mana_cost: editing_ability.mana_cost,
                ability_types,
                unlock_requirement,
            };

            editor_state.status_message = format!("Saving ability: {}...", ability_data.name);
            update_ability_events.write(api_events::UpdateAbilityEvent { ability: ability_data });
        }
    }

    if editor_state.action_delete_ability {
        editor_state.action_delete_ability = false;

        if let Some(ability_id) = editor_state.abilities.selected_ability {
            editor_state.status_message = format!("Deleting ability {}...", ability_id);
            delete_ability_events.write(api_events::DeleteAbilityEvent { ability_id });
        }
    }

    // Loot Table Actions
    if editor_state.action_load_loot_tables {
        editor_state.action_load_loot_tables = false;
        editor_state.status_message = "Loading loot tables...".to_string();
        load_loot_table_events.write(api_events::LoadLootTableListEvent);
    }

    if editor_state.action_create_loot_table {
        editor_state.action_create_loot_table = false;

        let loot_table_name = editor_state.loot.new_loot_table_name.trim().to_string();
        if loot_table_name.is_empty() {
            editor_state.status_message = "Loot table name cannot be empty".to_string();
            return;
        }

        let loot_table_id = loot_table_name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        let loot_table_data = api_events::LootTableData {
            id: loot_table_id,
            name: loot_table_name,
            table_type: editor_state.loot.new_loot_table_type.clone(),
        };

        editor_state.status_message = format!("Creating loot table: {}...", loot_table_data.name);
        create_loot_table_events.write(api_events::CreateLootTableEvent { loot_table: loot_table_data });

        editor_state.loot.new_loot_table_name.clear();
        editor_state.loot.new_loot_table_type.clear();
        editor_state.loot.show_create_dialog = false;
    }

    if editor_state.action_save_loot_table {
        editor_state.action_save_loot_table = false;

        if let Some(ref editing_loot_table) = editor_state.loot.editing_loot_table {
            let loot_table_data = api_events::LootTableData {
                id: editing_loot_table.id.clone(),
                name: editing_loot_table.name.clone(),
                table_type: editing_loot_table.table_type.clone(),
            };

            editor_state.status_message = format!("Saving loot table: {}...", loot_table_data.name);
            update_loot_table_events.write(api_events::UpdateLootTableEvent { loot_table: loot_table_data });
        }
    }

    if editor_state.action_delete_loot_table {
        editor_state.action_delete_loot_table = false;

        if let Some(loot_table_id) = editor_state.loot.selected_loot_table.clone() {
            editor_state.status_message = format!("Deleting loot table {}...", loot_table_id);
            delete_loot_table_events.write(api_events::DeleteLootTableEvent { loot_table_id });
        }
    }
}

// =============================================================================
// Tile Texture Loading Systems
// =============================================================================

/// System to start loading tile textures via AssetServer when palette is ready
fn load_tile_textures(
    asset_server: Res<AssetServer>,
    mut editor_state: ResMut<EditorState>,
) {
    // Only load once when textures_loading is true and handles are empty
    if !editor_state.world.tile_palette.textures_loading {
        return;
    }
    if !editor_state.world.tile_palette.texture_handles.is_empty() {
        return; // Already started loading
    }

    info!("Starting tile texture loading...");

    // Collect tile info first to avoid borrow checker issues
    let tiles_to_load: Vec<(u32, String)> = editor_state.world.tile_palette.ground_tiles
        .iter()
        .chain(editor_state.world.tile_palette.decoration_tiles.iter())
        .map(|tile| (tile.id, tile.path.clone()))
        .collect();

    // Now load textures and insert handles
    for (tile_id, path) in tiles_to_load {
        let handle = asset_server.load::<Image>(&path);
        editor_state.world.tile_palette.texture_handles.insert(tile_id, handle);
    }

    info!("Initiated loading of {} tile textures", editor_state.world.tile_palette.texture_handles.len());
}

/// System to register loaded textures with egui for display in the palette
fn register_tile_textures(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    images: Res<Assets<Image>>,
) {
    // Only run when textures are loading but not yet registered
    if !editor_state.world.tile_palette.textures_loading {
        return;
    }
    if editor_state.world.tile_palette.textures_registered {
        return;
    }
    if editor_state.world.tile_palette.texture_handles.is_empty() {
        return; // Handles not yet created
    }

    // Check if at least some images are loaded (for progressive loading)
    // We'll register what we can and continue checking each frame
    let mut any_registered = false;
    let handles_to_check: Vec<_> = editor_state.world.tile_palette.texture_handles
        .iter()
        .filter(|(id, _)| !editor_state.world.tile_palette.egui_texture_ids.contains_key(id))
        .map(|(id, handle)| (*id, handle.clone()))
        .collect();

    for (tile_id, handle) in handles_to_check {
        // Check if this image is loaded
        if images.get(&handle).is_some() {
            // Register with egui using bevy_egui's add_image
            // Wrap handle in EguiTextureHandle::Strong for bevy_egui 0.38
            let egui_handle = bevy_egui::EguiTextureHandle::Strong(handle);
            let egui_id = contexts.add_image(egui_handle);
            editor_state.world.tile_palette.egui_texture_ids.insert(tile_id, egui_id);
            any_registered = true;
        }
    }

    // Check if all textures are now registered
    let total_tiles = editor_state.world.tile_palette.texture_handles.len();
    let registered_tiles = editor_state.world.tile_palette.egui_texture_ids.len();

    if registered_tiles == total_tiles {
        editor_state.world.tile_palette.textures_registered = true;
        editor_state.world.tile_palette.textures_loading = false;
        info!("All {} tile textures registered with egui", registered_tiles);
    } else if any_registered {
        // Progress update
        info!("Tile texture progress: {}/{} registered", registered_tiles, total_tiles);
    }
}

// =============================================================================
// New Tileset System
// =============================================================================

/// Initialize the hybrid tileset system with known tilesets
fn initialize_tilesets(editor_state: &mut EditorState) {
    // Ground tileset - grass, paths, water
    let ground_tileset = TilesetDefinition {
        id: "ground".to_string(),
        name: "Ground Tiles".to_string(),
        category: TileCategory::Ground,
        display_tile_size: 32,
        sources: vec![
            TileSource::SingleImage {
                path: "tiles/Tiles/Grass/Grass_1_Middle.png".to_string(),
                tile_index: 0,
                name: "Grass 1".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Grass/Grass_2_Middle.png".to_string(),
                tile_index: 0,
                name: "Grass 2".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Grass/Grass_3_Middle.png".to_string(),
                tile_index: 0,
                name: "Grass 3".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Grass/Grass_4_Middle.png".to_string(),
                tile_index: 0,
                name: "Grass 4".to_string(),
                has_collision: false,
            },
            TileSource::Spritesheet {
                path: "tiles/Tiles/Grass/Grass_Tiles_2.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                margin: 0,
                spacing: 0,
                image_width: 256,
                image_height: 160,
                columns: 16,
                rows: 10,
                first_tile_index: 0,
            },
            TileSource::Spritesheet {
                path: "tiles/Tiles/Grass/Grass_Tiles_3.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                margin: 0,
                spacing: 0,
                image_width: 256,
                image_height: 160,
                columns: 16,
                rows: 10,
                first_tile_index: 0,
            },
            TileSource::Spritesheet {
                path: "tiles/Tiles/Grass/Grass_Tiles_4.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                margin: 0,
                spacing: 0,
                image_width: 256,
                image_height: 160,
                columns: 16,
                rows: 10,
                first_tile_index: 0,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Grass/Path_Middle.png".to_string(),
                tile_index: 0,
                name: "Path Middle".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Grass/Path_Decoration.png".to_string(),
                tile_index: 0,
                name: "Path Decoration".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Cobble_Road/Cobble_Road_1.png".to_string(),
                tile_index: 0,
                name: "Cobble Road 1".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Cobble_Road/Cobble_Road_2.png".to_string(),
                tile_index: 0,
                name: "Cobble Road 2".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Pavement_Tiles.png".to_string(),
                tile_index: 0,
                name: "Pavement".to_string(),
                has_collision: false,
            },
        ],
        total_tiles: 0,
        tile_metadata: std::collections::HashMap::new(),
        terrain_sets: Vec::new(),
    };

    // Water tileset
    let water_tileset = TilesetDefinition {
        id: "water".to_string(),
        name: "Water Tiles".to_string(),
        category: TileCategory::Ground,
        display_tile_size: 32,
        sources: vec![
            TileSource::SingleImage {
                path: "tiles/Tiles/Water/Water_Middle.png".to_string(),
                tile_index: 0,
                name: "Water Middle".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Water/Water_Tile_1.png".to_string(),
                tile_index: 0,
                name: "Water Tile 1".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Water/Water_Tile_2.png".to_string(),
                tile_index: 0,
                name: "Water Tile 2".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Water/Water_Tile_3.png".to_string(),
                tile_index: 0,
                name: "Water Tile 3".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Tiles/Water/Water_Tile_4.png".to_string(),
                tile_index: 0,
                name: "Water Tile 4".to_string(),
                has_collision: false,
            },
            TileSource::Spritesheet {
                path: "tiles/Tiles/Beach/Beach_Tiles.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                margin: 0,
                spacing: 0,
                image_width: 480,
                image_height: 48,
                columns: 30,
                rows: 3,
                first_tile_index: 0,
            },
        ],
        total_tiles: 0,
        tile_metadata: std::collections::HashMap::new(),
        terrain_sets: Vec::new(),
    };

    // Trees tileset
    let trees_tileset = TilesetDefinition {
        id: "trees".to_string(),
        name: "Trees".to_string(),
        category: TileCategory::Decorations,
        display_tile_size: 48,
        sources: vec![
            TileSource::SingleImage {
                path: "tiles/Trees/Big_Oak_Tree.png".to_string(),
                tile_index: 0,
                name: "Big Oak Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Big_Birch_Tree.png".to_string(),
                tile_index: 0,
                name: "Big Birch Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Big_Spruce_tree.png".to_string(),
                tile_index: 0,
                name: "Big Spruce Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Big_Fruit_Tree.png".to_string(),
                tile_index: 0,
                name: "Big Fruit Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Medium_Oak_Tree.png".to_string(),
                tile_index: 0,
                name: "Medium Oak Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Medium_Birch_Tree.png".to_string(),
                tile_index: 0,
                name: "Medium Birch Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Medium_Spruce_Tree.png".to_string(),
                tile_index: 0,
                name: "Medium Spruce Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Medium_Fruit_Tree.png".to_string(),
                tile_index: 0,
                name: "Medium Fruit Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Small_Oak_Tree.png".to_string(),
                tile_index: 0,
                name: "Small Oak Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Small_Birch_Tree.png".to_string(),
                tile_index: 0,
                name: "Small Birch Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Small_Spruce_Tree.png".to_string(),
                tile_index: 0,
                name: "Small Spruce Tree".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Trees/Small_Fruit_Tree.png".to_string(),
                tile_index: 0,
                name: "Small Fruit Tree".to_string(),
                has_collision: true,
            },
        ],
        total_tiles: 0,
        tile_metadata: std::collections::HashMap::new(),
        terrain_sets: Vec::new(),
    };

    // Outdoor decorations tileset
    let outdoor_decor_tileset = TilesetDefinition {
        id: "outdoor_decor".to_string(),
        name: "Outdoor Decorations".to_string(),
        category: TileCategory::Decorations,
        display_tile_size: 32,
        sources: vec![
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Flowers.png".to_string(),
                tile_index: 0,
                name: "Flowers".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Fountain.png".to_string(),
                tile_index: 0,
                name: "Fountain".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Well.png".to_string(),
                tile_index: 0,
                name: "Well".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Benches.png".to_string(),
                tile_index: 0,
                name: "Benches".to_string(),
                has_collision: false,
            },
            TileSource::Spritesheet {
                path: "tiles/Outdoor decoration/Fences.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                margin: 0,
                spacing: 0,
                image_width: 64,
                image_height: 64,
                columns: 4,
                rows: 4,
                first_tile_index: 0,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Fence_Big.png".to_string(),
                tile_index: 0,
                name: "Fence Big".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/barrels.png".to_string(),
                tile_index: 0,
                name: "Barrels".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Hay_Bales.png".to_string(),
                tile_index: 0,
                name: "Hay Bales".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Lanter_Posts.png".to_string(),
                tile_index: 0,
                name: "Lantern Posts".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Camp_Decor.png".to_string(),
                tile_index: 0,
                name: "Camp Decor".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Outdoor_Decor.png".to_string(),
                tile_index: 0,
                name: "Outdoor Decor".to_string(),
                has_collision: false,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Signs.png".to_string(),
                tile_index: 0,
                name: "Signs".to_string(),
                has_collision: true,
            },
            TileSource::SingleImage {
                path: "tiles/Outdoor decoration/Boat.png".to_string(),
                tile_index: 0,
                name: "Boat".to_string(),
                has_collision: true,
            },
        ],
        total_tiles: 0,
        tile_metadata: std::collections::HashMap::new(),
        terrain_sets: Vec::new(),
    };

    // Add tilesets and recalculate indices
    let mut tilesets = vec![ground_tileset, water_tileset, trees_tileset, outdoor_decor_tileset];
    for tileset in &mut tilesets {
        tileset.recalculate_indices();
    }

    let tileset_count = tilesets.len();
    let total_sources: usize = tilesets.iter().map(|t| t.sources.len()).sum();

    editor_state.world.tile_palette.tilesets = tilesets;
    editor_state.world.tile_palette.selected_tileset = Some(0);
    editor_state.world.tile_palette.tileset_textures_loading = true;

    info!("Initialized {} tilesets with {} total sources", tileset_count, total_sources);
}

/// System to load tileset textures via AssetServer
fn load_tileset_textures(
    asset_server: Res<AssetServer>,
    mut editor_state: ResMut<EditorState>,
) {
    if !editor_state.world.tile_palette.tileset_textures_loading {
        return;
    }

    // Collect unique image paths from all tileset sources
    let mut paths_needed: Vec<String> = Vec::new();

    for tileset in &editor_state.world.tile_palette.tilesets {
        for source in &tileset.sources {
            let path = source.image_path().to_string();
            if !paths_needed.contains(&path) {
                paths_needed.push(path);
            }
        }
    }

    // Find paths that are NOT yet in our handles (need to be loaded)
    let missing_paths: Vec<String> = paths_needed
        .iter()
        .filter(|path| !editor_state.world.tile_palette.tileset_texture_handles.contains_key(*path))
        .cloned()
        .collect();

    // If no missing paths, nothing to do
    if missing_paths.is_empty() {
        return;
    }

    info!("Loading {} tileset textures...", missing_paths.len());

    // Load each missing path
    for path in &missing_paths {
        let handle = asset_server.load::<Image>(path);
        editor_state.world.tile_palette.tileset_texture_handles.insert(path.clone(), handle);
    }

    info!("Initiated loading of {} tileset textures (total: {})",
        missing_paths.len(),
        editor_state.world.tile_palette.tileset_texture_handles.len());
}

/// System to register tileset textures with egui
fn register_tileset_textures(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    images: Res<Assets<Image>>,
) {
    if !editor_state.world.tile_palette.tileset_textures_loading {
        return;
    }
    if editor_state.world.tile_palette.tileset_texture_handles.is_empty() {
        return; // Handles not yet created
    }

    let mut any_registered = false;
    let handles_to_check: Vec<_> = editor_state.world.tile_palette.tileset_texture_handles
        .iter()
        .filter(|(path, _)| !editor_state.world.tile_palette.tileset_egui_ids.contains_key(*path))
        .map(|(path, handle)| (path.clone(), handle.clone()))
        .collect();

    for (path, handle) in handles_to_check {
        if let Some(image) = images.get(&handle) {
            // Get image dimensions
            let image_width = image.width();
            let image_height = image.height();

            // Update tileset source metadata with actual image dimensions
            update_tileset_source_dimensions(&mut editor_state, &path, image_width, image_height);

            let egui_handle = bevy_egui::EguiTextureHandle::Strong(handle);
            let egui_id = contexts.add_image(egui_handle);
            editor_state.world.tile_palette.tileset_egui_ids.insert(path, egui_id);
            any_registered = true;
        }
    }

    let total = editor_state.world.tile_palette.tileset_texture_handles.len();
    let registered = editor_state.world.tile_palette.tileset_egui_ids.len();

    if registered == total {
        editor_state.world.tile_palette.tileset_textures_loading = false;
        info!("All {} tileset textures registered with egui", registered);
    } else if any_registered {
        info!("Tileset texture progress: {}/{} registered", registered, total);
    }
}

/// Update the tileset source dimensions when image is loaded
fn update_tileset_source_dimensions(
    editor_state: &mut EditorState,
    path: &str,
    image_width: u32,
    image_height: u32,
) {
    use crate::editor_state::TileSource;

    for tileset in &mut editor_state.world.tile_palette.tilesets {
        for source in &mut tileset.sources {
            if let TileSource::Spritesheet {
                path: source_path,
                tile_width,
                tile_height,
                margin,
                spacing,
                image_width: ref mut iw,
                image_height: ref mut ih,
                columns: ref mut cols,
                rows: ref mut rws,
                ..
            } = source {
                if source_path == path && (*cols == 0 || *rws == 0) {
                    *iw = image_width;
                    *ih = image_height;

                    // Calculate columns and rows based on tile size, margin, and spacing
                    // Formula: columns = (image_width - 2*margin + spacing) / (tile_width + spacing)
                    let tw = *tile_width;
                    let th = *tile_height;
                    let m = *margin;
                    let s = *spacing;

                    if tw > 0 && th > 0 {
                        // Account for margin and spacing
                        let usable_width = image_width.saturating_sub(2 * m);
                        let usable_height = image_height.saturating_sub(2 * m);

                        *cols = if s > 0 {
                            (usable_width + s) / (tw + s)
                        } else {
                            usable_width / tw
                        };

                        *rws = if s > 0 {
                            (usable_height + s) / (th + s)
                        } else {
                            usable_height / th
                        };

                        // Update total tiles
                        tileset.total_tiles = *cols * *rws;

                        info!(
                            "Updated tileset source '{}': {}x{} image, {}x{} tiles, {} total",
                            path, image_width, image_height, *cols, *rws, tileset.total_tiles
                        );
                    }
                }
            }
        }
    }
}

/// System to update tileset dimensions for ALL spritesheet sources
/// This runs independently from registration to ensure dimensions are calculated
/// even when textures are already registered (e.g., reusing same image path)
fn update_tileset_dimensions(
    mut editor_state: ResMut<EditorState>,
    images: Res<Assets<Image>>,
) {
    use crate::editor_state::TileSource;

    // Only run if we have texture handles loaded
    if editor_state.world.tile_palette.tileset_texture_handles.is_empty() {
        return;
    }

    // Track if we made any changes
    let mut any_updated = false;

    // Get handles clone to avoid borrow issues
    let handles: Vec<_> = editor_state.world.tile_palette.tileset_texture_handles
        .iter()
        .map(|(path, handle)| (path.clone(), handle.clone()))
        .collect();

    // Check each tileset source
    for tileset in &mut editor_state.world.tile_palette.tilesets {
        for source in &mut tileset.sources {
            if let TileSource::Spritesheet {
                path: source_path,
                tile_width,
                tile_height,
                margin,
                spacing,
                image_width: ref mut iw,
                image_height: ref mut ih,
                columns: ref mut cols,
                rows: ref mut rws,
                ..
            } = source {
                // Only update if dimensions are not yet calculated
                if *cols == 0 || *rws == 0 {
                    // Look up the image handle for this path
                    if let Some((_, handle)) = handles.iter().find(|(p, _)| p == source_path) {
                        if let Some(image) = images.get(handle) {
                            let img_w = image.width();
                            let img_h = image.height();

                            *iw = img_w;
                            *ih = img_h;

                            let tw = *tile_width;
                            let th = *tile_height;
                            let m = *margin;
                            let s = *spacing;

                            if tw > 0 && th > 0 {
                                let usable_width = img_w.saturating_sub(2 * m);
                                let usable_height = img_h.saturating_sub(2 * m);

                                *cols = if s > 0 {
                                    (usable_width + s) / (tw + s)
                                } else {
                                    usable_width / tw
                                };

                                *rws = if s > 0 {
                                    (usable_height + s) / (th + s)
                                } else {
                                    usable_height / th
                                };

                                info!(
                                    "Dimension update for '{}': {}x{} image -> {}x{} grid",
                                    source_path, img_w, img_h, *cols, *rws
                                );
                                any_updated = true;
                            }
                        }
                    }
                }
            }
        }

        // Recalculate total tiles for the tileset if we updated anything
        if any_updated {
            tileset.recalculate_indices();
        }
    }
}
