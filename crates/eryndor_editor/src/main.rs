//! Eryndor Game Content Editor
//! A web-based design toolkit for creating game content.

mod api_client;
mod api_events;
mod editor_state;
mod modules;
mod ui;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

use api_events::ApiEventsPlugin;
use editor_state::{EditorState, EditorTab};
use ui::{render_main_menu, render_tab_bar, render_status_bar};

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
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn 2D camera for the editor
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.12)),
            ..default()
        },
    ));

    info!("Eryndor Editor initialized");
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

    // Main content area based on active tab
    egui::CentralPanel::default().show(ctx, |ui| {
        match editor_state.active_tab {
            EditorTab::World => modules::world::render(ui, &mut editor_state),
            EditorTab::Items => modules::items::render(ui, &mut editor_state),
            EditorTab::Enemies => modules::enemies::render(ui, &mut editor_state),
            EditorTab::Npcs => modules::npcs::render(ui, &mut editor_state),
            EditorTab::Quests => modules::quests::render(ui, &mut editor_state),
            EditorTab::Abilities => modules::abilities::render(ui, &mut editor_state),
            EditorTab::Loot => modules::loot::render(ui, &mut editor_state),
            EditorTab::Assets => modules::assets::render(ui, &mut editor_state),
        }
    });

    // Status bar at bottom
    render_status_bar(ctx, &editor_state);
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
        };

        editor_state.status_message = format!("Creating enemy: {}...", enemy_data.name);
        create_enemy_events.write(api_events::CreateEnemyEvent { enemy: enemy_data });

        editor_state.enemies.new_enemy_name.clear();
        editor_state.enemies.show_create_dialog = false;
    }

    if editor_state.action_save_enemy {
        editor_state.action_save_enemy = false;

        if let Some(ref editing_enemy) = editor_state.enemies.editing_enemy {
            let enemy_data = api_events::EnemyData {
                id: editing_enemy.id,
                name: editing_enemy.name.clone(),
                max_health: editing_enemy.max_health,
                attack_power: editing_enemy.attack_power,
                defense: editing_enemy.defense,
                move_speed: editing_enemy.move_speed,
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
