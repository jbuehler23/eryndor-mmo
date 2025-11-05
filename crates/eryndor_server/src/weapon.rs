use eryndor_shared::*;

/// Weapon types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Sword,
    Dagger,
    Staff,
    Mace,
    Bow,
    Axe,
}

/// Stats for a weapon type
#[derive(Debug, Clone, Copy)]
pub struct WeaponStats {
    pub weapon_type: WeaponType,
    pub attack_speed: f32,  // Attacks per second
    pub range: f32,         // Attack range in pixels
    pub damage_multiplier: f32, // Multiplier for base damage
}

impl WeaponType {
    /// Get the stats for this weapon type
    pub fn stats(&self) -> WeaponStats {
        match self {
            WeaponType::Sword => WeaponStats {
                weapon_type: WeaponType::Sword,
                attack_speed: 1.0,
                range: 30.0,
                damage_multiplier: 1.0,
            },
            WeaponType::Dagger => WeaponStats {
                weapon_type: WeaponType::Dagger,
                attack_speed: 1.5,
                range: 30.0,
                damage_multiplier: 0.7,
            },
            WeaponType::Staff => WeaponStats {
                weapon_type: WeaponType::Staff,
                attack_speed: 0.7,
                range: 200.0,
                damage_multiplier: 0.8,
            },
            WeaponType::Mace => WeaponStats {
                weapon_type: WeaponType::Mace,
                attack_speed: 0.8,
                range: 30.0,
                damage_multiplier: 1.2,
            },
            WeaponType::Bow => WeaponStats {
                weapon_type: WeaponType::Bow,
                attack_speed: 1.2,
                range: 250.0,
                damage_multiplier: 0.9,
            },
            WeaponType::Axe => WeaponStats {
                weapon_type: WeaponType::Axe,
                attack_speed: 0.9,
                range: 30.0,
                damage_multiplier: 1.1,
            },
        }
    }

    /// Get weapon type from item ID
    pub fn from_item_id(item_id: u32) -> Option<WeaponType> {
        match item_id {
            ITEM_SWORD => Some(WeaponType::Sword),
            ITEM_DAGGER => Some(WeaponType::Dagger),
            ITEM_WAND => Some(WeaponType::Staff), // Wand uses staff proficiency
            _ => None,
        }
    }
}

/// Get starting weapon proficiencies for a class
pub fn get_starting_proficiencies(class: CharacterClass) -> Vec<(WeaponType, u32)> {
    match class {
        CharacterClass::Knight => vec![
            (WeaponType::Sword, 10),
            (WeaponType::Mace, 5),
            (WeaponType::Axe, 5),
        ],
        CharacterClass::Mage => vec![
            (WeaponType::Staff, 10),
            (WeaponType::Dagger, 5),
        ],
        CharacterClass::Rogue => vec![
            (WeaponType::Dagger, 10),
            (WeaponType::Bow, 5),
            (WeaponType::Sword, 5),
        ],
    }
}
