//! Quest log persistence.
//!
//! Handles saving and loading quest progress.

use sqlx::{SqlitePool, Row};
use tracing::info;
use eryndor_shared::*;

/// Load quest log for a character
pub async fn load_quest_log(pool: &SqlitePool, character_id: i64) -> Result<QuestLog, String> {
    let result = sqlx::query(
        "SELECT quest_id, completed, progress FROM character_quests WHERE character_id = ?1"
    )
    .bind(character_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => {
            let mut quest_log = QuestLog::default();

            for row in rows {
                let quest_id: i32 = row.get(0);
                let completed: i32 = row.get(1);
                let progress_json: Option<String> = row.get(2);

                if completed == 1 {
                    quest_log.completed_quests.insert(quest_id as u32);
                } else {
                    let progress: Vec<u32> = if let Some(json_str) = progress_json {
                        serde_json::from_str(&json_str).unwrap_or_default()
                    } else {
                        Vec::new()
                    };

                    quest_log.active_quests.push(ActiveQuest {
                        quest_id: quest_id as u32,
                        progress,
                    });
                }
            }

            info!(
                "Loaded {} active and {} completed quests for character {}",
                quest_log.active_quests.len(),
                quest_log.completed_quests.len(),
                character_id
            );
            Ok(quest_log)
        }
        Err(e) => Err(format!("Failed to load quest log: {}", e)),
    }
}

/// Save quest log for a character
pub async fn save_quest_log(
    pool: &SqlitePool,
    character_id: i64,
    quest_log: &QuestLog,
) -> Result<(), String> {
    let total_quests = quest_log.active_quests.len() + quest_log.completed_quests.len();
    info!(
        "Saving quest log for character {} ({} active, {} completed)",
        character_id,
        quest_log.active_quests.len(),
        quest_log.completed_quests.len()
    );

    // Delete all existing quests for this character
    let delete_result = sqlx::query("DELETE FROM character_quests WHERE character_id = ?1")
        .bind(character_id)
        .execute(pool)
        .await;

    if let Err(e) = delete_result {
        return Err(format!("Failed to clear quest log: {}", e));
    }

    // Insert active quests
    for active_quest in &quest_log.active_quests {
        let progress_json =
            serde_json::to_string(&active_quest.progress).unwrap_or_else(|_| "[]".to_string());

        let insert_result = sqlx::query(
            "INSERT INTO character_quests (character_id, quest_id, completed, progress)
             VALUES (?1, ?2, 0, ?3)"
        )
        .bind(character_id)
        .bind(active_quest.quest_id as i32)
        .bind(progress_json)
        .execute(pool)
        .await;

        if let Err(e) = insert_result {
            return Err(format!(
                "Failed to save active quest {}: {}",
                active_quest.quest_id, e
            ));
        }
    }

    // Insert completed quests
    for &quest_id in &quest_log.completed_quests {
        let insert_result = sqlx::query(
            "INSERT INTO character_quests (character_id, quest_id, completed, progress)
             VALUES (?1, ?2, 1, NULL)"
        )
        .bind(character_id)
        .bind(quest_id as i32)
        .execute(pool)
        .await;

        if let Err(e) = insert_result {
            return Err(format!("Failed to save completed quest {}: {}", quest_id, e));
        }
    }

    info!(
        "Successfully saved {} quests for character {}",
        total_quests, character_id
    );
    Ok(())
}
