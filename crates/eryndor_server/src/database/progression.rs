//! Character progression persistence.
//!
//! Handles experience, weapon proficiency, and armor proficiency saving/loading.

use sqlx::{SqlitePool, Row};
use eryndor_shared::*;

/// Load all progression data for a character
pub async fn load_progression(
    pool: &SqlitePool,
    character_id: i64,
) -> Result<
    (
        Experience,
        WeaponProficiency,
        WeaponProficiencyExp,
        ArmorProficiency,
        ArmorProficiencyExp,
        UnlockedArmorPassives,
    ),
    String,
> {
    let result = sqlx::query(
        "SELECT current_xp,
                weapon_prof_sword, weapon_prof_dagger, weapon_prof_staff, weapon_prof_mace, weapon_prof_bow, weapon_prof_axe,
                weapon_exp_sword, weapon_exp_dagger, weapon_exp_staff, weapon_exp_mace, weapon_exp_bow, weapon_exp_axe,
                armor_prof_light, armor_prof_medium, armor_prof_heavy,
                armor_exp_light, armor_exp_medium, armor_exp_heavy,
                unlocked_armor_passives
         FROM characters WHERE id = ?1"
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            // Experience
            let current_xp: i32 = row.try_get(0).unwrap_or(0);

            // Get character level for Experience initialization
            let level_result = sqlx::query("SELECT level FROM characters WHERE id = ?1")
                .bind(character_id)
                .fetch_one(pool)
                .await;

            let level = match level_result {
                Ok(row) => row.get::<i32, _>(0) as u32,
                Err(_) => 1,
            };

            let mut experience = Experience::new(level);
            experience.current_xp = current_xp as u32;

            // Weapon Proficiency
            let weapon_prof = WeaponProficiency {
                sword: row.try_get(1).unwrap_or(0),
                dagger: row.try_get(2).unwrap_or(0),
                staff: row.try_get(3).unwrap_or(0),
                wand: 1, // Default wand proficiency (not in DB yet)
                mace: row.try_get(4).unwrap_or(0),
                bow: row.try_get(5).unwrap_or(0),
                axe: row.try_get(6).unwrap_or(0),
            };

            // Weapon Proficiency Experience
            let weapon_exp = WeaponProficiencyExp {
                sword_xp: row.try_get(7).unwrap_or(0),
                dagger_xp: row.try_get(8).unwrap_or(0),
                staff_xp: row.try_get(9).unwrap_or(0),
                wand_xp: 0, // Default wand XP (not in DB yet)
                mace_xp: row.try_get(10).unwrap_or(0),
                bow_xp: row.try_get(11).unwrap_or(0),
                axe_xp: row.try_get(12).unwrap_or(0),
            };

            // Armor Proficiency
            let armor_prof = ArmorProficiency {
                light: row.try_get(13).unwrap_or(0),
                medium: row.try_get(14).unwrap_or(0),
                heavy: row.try_get(15).unwrap_or(0),
            };

            // Armor Proficiency Experience
            let armor_exp = ArmorProficiencyExp {
                light_xp: row.try_get(16).unwrap_or(0),
                medium_xp: row.try_get(17).unwrap_or(0),
                heavy_xp: row.try_get(18).unwrap_or(0),
            };

            // Unlocked Armor Passives
            let passives_json: Option<String> = row.try_get(19).ok().flatten();
            let unlocked_passives = if let Some(json_str) = passives_json {
                match serde_json::from_str(&json_str) {
                    Ok(passives) => passives,
                    Err(_) => UnlockedArmorPassives::default(),
                }
            } else {
                UnlockedArmorPassives::default()
            };

            Ok((
                experience,
                weapon_prof,
                weapon_exp,
                armor_prof,
                armor_exp,
                unlocked_passives,
            ))
        }
        Ok(None) => Err("Character not found".to_string()),
        Err(e) => Err(format!("Failed to load progression: {}", e)),
    }
}

/// Save all progression data for a character
pub async fn save_progression(
    pool: &SqlitePool,
    character_id: i64,
    character_level: u32,
    experience: &Experience,
    weapon_prof: &WeaponProficiency,
    weapon_exp: &WeaponProficiencyExp,
    armor_prof: &ArmorProficiency,
    armor_exp: &ArmorProficiencyExp,
    unlocked_passives: &UnlockedArmorPassives,
) -> Result<(), String> {
    let passives_json =
        serde_json::to_string(&unlocked_passives.passives).unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query(
        "UPDATE characters SET
            level = ?1,
            current_xp = ?2,
            weapon_prof_sword = ?3, weapon_prof_dagger = ?4, weapon_prof_staff = ?5,
            weapon_prof_mace = ?6, weapon_prof_bow = ?7, weapon_prof_axe = ?8,
            weapon_exp_sword = ?9, weapon_exp_dagger = ?10, weapon_exp_staff = ?11,
            weapon_exp_mace = ?12, weapon_exp_bow = ?13, weapon_exp_axe = ?14,
            armor_prof_light = ?15, armor_prof_medium = ?16, armor_prof_heavy = ?17,
            armor_exp_light = ?18, armor_exp_medium = ?19, armor_exp_heavy = ?20,
            unlocked_armor_passives = ?21
         WHERE id = ?22"
    )
    .bind(character_level as i32)
    .bind(experience.current_xp as i32)
    .bind(weapon_prof.sword)
    .bind(weapon_prof.dagger)
    .bind(weapon_prof.staff)
    .bind(weapon_prof.mace)
    .bind(weapon_prof.bow)
    .bind(weapon_prof.axe)
    .bind(weapon_exp.sword_xp)
    .bind(weapon_exp.dagger_xp)
    .bind(weapon_exp.staff_xp)
    .bind(weapon_exp.mace_xp)
    .bind(weapon_exp.bow_xp)
    .bind(weapon_exp.axe_xp)
    .bind(armor_prof.light)
    .bind(armor_prof.medium)
    .bind(armor_prof.heavy)
    .bind(armor_exp.light_xp)
    .bind(armor_exp.medium_xp)
    .bind(armor_exp.heavy_xp)
    .bind(passives_json)
    .bind(character_id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save progression: {}", e)),
    }
}
