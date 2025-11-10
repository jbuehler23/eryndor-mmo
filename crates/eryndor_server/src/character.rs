use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use avian2d::prelude::{RigidBody, Collider, CollisionLayers};
use crate::{PhysicsPosition, PhysicsVelocity};

/// Get class-specific combat stats
pub fn get_class_stats(class: CharacterClass) -> CombatStats {
    match class {
        CharacterClass::Knight => CombatStats {
            attack_power: 10.0,
            defense: 8.0,
            crit_chance: 0.05,
        },
        CharacterClass::Mage => CombatStats {
            attack_power: 8.0,
            defense: 2.0,
            crit_chance: 0.10,
        },
        CharacterClass::Rogue => CombatStats {
            attack_power: 12.0,
            defense: 3.0,
            crit_chance: 0.15,
        },
    }
}

/// Get class-specific health and mana
pub fn get_class_health_mana(class: CharacterClass) -> (Health, Mana) {
    match class {
        CharacterClass::Knight => (
            Health { current: 120.0, max: 120.0 },
            Mana { current: 80.0, max: 80.0 },
        ),
        CharacterClass::Mage => (
            Health { current: 60.0, max: 60.0 },
            Mana { current: 150.0, max: 150.0 },
        ),
        CharacterClass::Rogue => (
            Health { current: 80.0, max: 80.0 },
            Mana { current: 100.0, max: 100.0 },
        ),
    }
}

/// Get class-specific regeneration rates
pub fn get_class_regen(class: CharacterClass) -> (HealthRegen, ManaRegen) {
    match class {
        CharacterClass::Knight => (
            HealthRegen {
                base_regen: 5.0,
                in_combat_multiplier: 0.0,  // Knights don't regen health in combat
            },
            ManaRegen {
                base_regen: 1.0,
                in_combat_multiplier: 0.5,  // 50% mana regen in combat
            },
        ),
        CharacterClass::Mage => (
            HealthRegen {
                base_regen: 3.0,  // Lower base health regen (squishier class)
                in_combat_multiplier: 0.0,
            },
            ManaRegen {
                base_regen: 3.0,  // High mana regen for casters
                in_combat_multiplier: 0.5,
            },
        ),
        CharacterClass::Rogue => (
            HealthRegen {
                base_regen: 4.0,  // Medium health regen
                in_combat_multiplier: 0.0,
            },
            ManaRegen {
                base_regen: 2.0,  // Medium mana regen
                in_combat_multiplier: 0.5,
            },
        ),
    }
}

/// Get class-specific visual shape
pub fn get_class_visual(class: CharacterClass) -> VisualShape {
    match class {
        CharacterClass::Rogue => VisualShape {
            shape_type: ShapeType::Triangle,
            color: COLOR_PLAYER,
            size: PLAYER_SIZE,
        },
        CharacterClass::Mage => VisualShape {
            shape_type: ShapeType::Circle,
            color: COLOR_PLAYER,
            size: PLAYER_SIZE,
        },
        CharacterClass::Knight => VisualShape {
            shape_type: ShapeType::Square,
            color: COLOR_PLAYER,
            size: PLAYER_SIZE,
        },
    }
}

/// Spawn a character entity with all required components
/// Returns the character entity ID
pub fn spawn_character_components(
    commands: &mut Commands,
    character: Character,
    position: Position,
    health: Health,
    mana: Mana,
    equipment: Equipment,
    inventory: Inventory,
    quest_log: QuestLog,
    client_entity: Entity,
    character_db_id: i64,
) -> Entity {
    let class = character.class;

    // Get class-specific stats
    let combat_stats = get_class_stats(class);
    let visual = get_class_visual(class);
    let (health_regen, mana_regen) = get_class_regen(class);

    // Grant class-based starting abilities
    let mut learned_abilities = LearnedAbilities::default();
    let mut hotbar = Hotbar::default();

    for (i, ability_id) in class.starting_abilities().iter().enumerate() {
        learned_abilities.learn(*ability_id);
        // Add to hotbar automatically
        if i < hotbar.slots.len() {
            hotbar.slots[i] = Some(HotbarSlot::Ability(*ability_id));
        }
    }

    // Set up weapon proficiencies based on class
    let mut proficiency = WeaponProficiency::default();
    for (weapon_type, level) in crate::weapon::get_starting_proficiencies(class) {
        match weapon_type {
            crate::weapon::WeaponType::Sword => proficiency.sword = level,
            crate::weapon::WeaponType::Dagger => proficiency.dagger = level,
            crate::weapon::WeaponType::Staff => proficiency.staff = level,
            crate::weapon::WeaponType::Wand => proficiency.wand = level,
            crate::weapon::WeaponType::Mace => proficiency.mace = level,
            crate::weapon::WeaponType::Bow => proficiency.bow = level,
            crate::weapon::WeaponType::Axe => proficiency.axe = level,
        }
    }

    // Get character level before moving the character value
    let char_level = character.level;

    // Spawn character entity (split to avoid bundle size limit)
    let character_entity = commands.spawn((
        Replicated,
        Player,
        character,
        position,
        Velocity::default(),
        MoveSpeed::default(),
        health,
        mana,
        health_regen,
        mana_regen,
        combat_stats,
        CurrentTarget::default(),
        InCombat(false),
    )).id();

    commands.entity(character_entity).insert((
        inventory,
        equipment,
        hotbar,
        learned_abilities,
    ));

    commands.entity(character_entity).insert((
        quest_log,
        AbilityCooldowns::default(),
        visual,
        OwnedBy(client_entity),
        crate::auth::CharacterDatabaseId(character_db_id),
    ));

    commands.entity(character_entity).insert((
        AutoAttack::default(),
        proficiency,
    ));

    // Progression components
    commands.entity(character_entity).insert((
        Experience::new(char_level),
        WeaponProficiencyExp::default(),
        ArmorProficiency::default(),
        ArmorProficiencyExp::default(),
        UnlockedArmorPassives::default(),
    ));

    // Physics components (separate insert to avoid bundle size limit)
    commands.entity(character_entity).insert((
        PhysicsPosition(position.0),
        PhysicsVelocity::default(),
        RigidBody::Dynamic,
        Collider::circle(PLAYER_SIZE / 2.0),
        CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Npc, GameLayer::Environment]),
    ));

    character_entity
}
