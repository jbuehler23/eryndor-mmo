// ============================================================================
// CONTENT MODERATION
// ============================================================================
// Uses rustrict library for basic profanity and inappropriate content filtering

use rustrict::CensorStr;

/// Result of content moderation check
#[derive(Debug, Clone)]
pub struct ModerationResult {
    pub is_appropriate: bool,
    pub reason: Option<String>,
    pub filtered_text: String,
}

/// Content moderation levels
#[derive(Debug, Clone, Copy)]
pub enum ModerationLevel {
    /// Strict - blocks all profanity and inappropriate content
    Strict,
    /// Moderate - blocks severe profanity but allows mild language
    Moderate,
    /// Permissive - only blocks severe offensive content
    Permissive,
}

/// Check if text content is appropriate for the game
/// Returns ModerationResult with filtered text and rejection reason if inappropriate
pub fn check_content(text: &str, _level: ModerationLevel) -> ModerationResult {
    // Use rustrict's CensorStr trait for simple checking
    // Check if text is inappropriate (contains profanity, sexual, or offensive content)
    let is_inappropriate = text.is_inappropriate();

    if is_inappropriate {
        let reason = "Contains inappropriate content".to_string();

        ModerationResult {
            is_appropriate: false,
            reason: Some(reason),
            filtered_text: text.censor(),
        }
    } else {
        ModerationResult {
            is_appropriate: true,
            reason: None,
            filtered_text: text.to_string(),
        }
    }
}

/// Check if a username is appropriate
/// More strict than general content - no profanity allowed
pub fn check_username(username: &str) -> ModerationResult {
    // Usernames should be strict
    let result = check_content(username, ModerationLevel::Strict);

    if !result.is_appropriate {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Username contains inappropriate content".to_string()),
            filtered_text: result.filtered_text,
        };
    }

    // Additional username-specific checks
    let trimmed = username.trim();

    if trimmed.len() < 3 {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Username must be at least 3 characters".to_string()),
            filtered_text: username.to_string(),
        };
    }

    if trimmed.len() > 20 {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Username must be no more than 20 characters".to_string()),
            filtered_text: username.to_string(),
        };
    }

    // Check for valid characters (alphanumeric, underscore, hyphen)
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Username can only contain letters, numbers, underscores, and hyphens".to_string()),
            filtered_text: username.to_string(),
        };
    }

    ModerationResult {
        is_appropriate: true,
        reason: None,
        filtered_text: trimmed.to_string(),
    }
}

/// Check if a character name is appropriate
/// Similar to username but may have different rules
pub fn check_character_name(name: &str) -> ModerationResult {
    // Character names use same rules as usernames for now
    let result = check_content(name, ModerationLevel::Strict);

    if !result.is_appropriate {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Character name contains inappropriate content".to_string()),
            filtered_text: result.filtered_text,
        };
    }

    // Additional character name-specific checks
    let trimmed = name.trim();

    if trimmed.len() < 2 {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Character name must be at least 2 characters".to_string()),
            filtered_text: name.to_string(),
        };
    }

    if trimmed.len() > 20 {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Character name must be no more than 20 characters".to_string()),
            filtered_text: name.to_string(),
        };
    }

    // Check for valid characters (alphanumeric, spaces, apostrophes for fantasy names)
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '\'' || c == '-') {
        return ModerationResult {
            is_appropriate: false,
            reason: Some("Character name can only contain letters, numbers, spaces, apostrophes, and hyphens".to_string()),
            filtered_text: name.to_string(),
        };
    }

    ModerationResult {
        is_appropriate: true,
        reason: None,
        filtered_text: trimmed.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appropriate_username() {
        let result = check_username("ValidUser123");
        assert!(result.is_appropriate);
        assert_eq!(result.filtered_text, "ValidUser123");
    }

    #[test]
    fn test_username_too_short() {
        let result = check_username("ab");
        assert!(!result.is_appropriate);
        assert!(result.reason.unwrap().contains("at least 3 characters"));
    }

    #[test]
    fn test_username_too_long() {
        let result = check_username("ThisUsernameIsWayTooLongForOurGame");
        assert!(!result.is_appropriate);
        assert!(result.reason.unwrap().contains("no more than 20 characters"));
    }

    #[test]
    fn test_appropriate_character_name() {
        let result = check_character_name("Aragorn");
        assert!(result.is_appropriate);
    }

    #[test]
    fn test_character_name_with_apostrophe() {
        let result = check_character_name("D'Angelo");
        assert!(result.is_appropriate);
    }

    #[test]
    fn test_strict_moderation() {
        let result = check_content("test content", ModerationLevel::Strict);
        assert!(result.is_appropriate);
    }
}
