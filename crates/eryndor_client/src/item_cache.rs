use bevy::prelude::*;
use std::collections::HashMap;
use eryndor_shared::*;

/// Item type enum (client-side mirror of server's ItemType)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemType {
    Weapon,
    Helmet,
    Chest,
    Legs,
    Boots,
    Consumable,
    QuestItem,
}

/// Client-side item database - mirrors server's ItemDatabase
/// This allows the client to display item information without network requests
#[derive(Resource)]
pub struct ClientItemDatabase {
    pub items: HashMap<u32, ClientItemInfo>,
}

impl Default for ClientItemDatabase {
    fn default() -> Self {
        let mut items = HashMap::new();

        // ========== WEAPONS ==========

        items.insert(ITEM_DAGGER, ClientItemInfo {
            id: ITEM_DAGGER,
            name: "Dagger".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 8.0,
                crit_chance: 0.10,
                ..Default::default()
            },
        });

        items.insert(ITEM_WAND, ClientItemInfo {
            id: ITEM_WAND,
            name: "Wand".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 12.0,
                max_mana: 20.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_SWORD, ClientItemInfo {
            id: ITEM_SWORD,
            name: "Sword".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 10.0,
                defense: 2.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_STAFF, ClientItemInfo {
            id: ITEM_STAFF,
            name: "Staff".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 14.0,
                max_mana: 30.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_MACE, ClientItemInfo {
            id: ITEM_MACE,
            name: "Mace".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 11.0,
                defense: 3.0,
                max_health: 15.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_BOW, ClientItemInfo {
            id: ITEM_BOW,
            name: "Bow".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 9.0,
                crit_chance: 0.15,
                ..Default::default()
            },
        });

        items.insert(ITEM_AXE, ClientItemInfo {
            id: ITEM_AXE,
            name: "Axe".to_string(),
            item_type: ItemType::Weapon,
            stat_bonuses: ClientStatBonuses {
                attack_power: 13.0,
                max_health: 10.0,
                crit_chance: 0.05,
                ..Default::default()
            },
        });

        // ========== HELMETS ==========

        items.insert(ITEM_LEATHER_CAP, ClientItemInfo {
            id: ITEM_LEATHER_CAP,
            name: "Leather Cap".to_string(),
            item_type: ItemType::Helmet,
            stat_bonuses: ClientStatBonuses {
                defense: 2.0,
                max_health: 5.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_HAT, ClientItemInfo {
            id: ITEM_CLOTH_HAT,
            name: "Cloth Hat".to_string(),
            item_type: ItemType::Helmet,
            stat_bonuses: ClientStatBonuses {
                defense: 1.0,
                max_mana: 10.0,
                attack_power: 2.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_HELM, ClientItemInfo {
            id: ITEM_IRON_HELM,
            name: "Iron Helm".to_string(),
            item_type: ItemType::Helmet,
            stat_bonuses: ClientStatBonuses {
                defense: 4.0,
                max_health: 10.0,
                ..Default::default()
            },
        });

        // ========== CHEST ==========

        items.insert(ITEM_LEATHER_TUNIC, ClientItemInfo {
            id: ITEM_LEATHER_TUNIC,
            name: "Leather Tunic".to_string(),
            item_type: ItemType::Chest,
            stat_bonuses: ClientStatBonuses {
                defense: 4.0,
                max_health: 10.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_ROBE, ClientItemInfo {
            id: ITEM_CLOTH_ROBE,
            name: "Cloth Robe".to_string(),
            item_type: ItemType::Chest,
            stat_bonuses: ClientStatBonuses {
                defense: 2.0,
                max_mana: 20.0,
                attack_power: 4.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_CHESTPLATE, ClientItemInfo {
            id: ITEM_IRON_CHESTPLATE,
            name: "Iron Chestplate".to_string(),
            item_type: ItemType::Chest,
            stat_bonuses: ClientStatBonuses {
                defense: 8.0,
                max_health: 20.0,
                ..Default::default()
            },
        });

        // ========== LEGS ==========

        items.insert(ITEM_LEATHER_PANTS, ClientItemInfo {
            id: ITEM_LEATHER_PANTS,
            name: "Leather Pants".to_string(),
            item_type: ItemType::Legs,
            stat_bonuses: ClientStatBonuses {
                defense: 3.0,
                max_health: 8.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_PANTS, ClientItemInfo {
            id: ITEM_CLOTH_PANTS,
            name: "Cloth Pants".to_string(),
            item_type: ItemType::Legs,
            stat_bonuses: ClientStatBonuses {
                defense: 1.5,
                max_mana: 15.0,
                attack_power: 3.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_GREAVES, ClientItemInfo {
            id: ITEM_IRON_GREAVES,
            name: "Iron Greaves".to_string(),
            item_type: ItemType::Legs,
            stat_bonuses: ClientStatBonuses {
                defense: 6.0,
                max_health: 15.0,
                ..Default::default()
            },
        });

        // ========== BOOTS ==========

        items.insert(ITEM_LEATHER_BOOTS, ClientItemInfo {
            id: ITEM_LEATHER_BOOTS,
            name: "Leather Boots".to_string(),
            item_type: ItemType::Boots,
            stat_bonuses: ClientStatBonuses {
                defense: 1.5,
                max_health: 5.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_SHOES, ClientItemInfo {
            id: ITEM_CLOTH_SHOES,
            name: "Cloth Shoes".to_string(),
            item_type: ItemType::Boots,
            stat_bonuses: ClientStatBonuses {
                defense: 0.5,
                max_mana: 10.0,
                attack_power: 1.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_BOOTS, ClientItemInfo {
            id: ITEM_IRON_BOOTS,
            name: "Iron Boots".to_string(),
            item_type: ItemType::Boots,
            stat_bonuses: ClientStatBonuses {
                defense: 3.0,
                max_health: 10.0,
                ..Default::default()
            },
        });

        Self { items }
    }
}

impl ClientItemDatabase {
    /// Get item name by ID, returns "Unknown Item" if not found
    pub fn get_item_name(&self, item_id: u32) -> String {
        self.items
            .get(&item_id)
            .map(|item| item.name.clone())
            .unwrap_or_else(|| format!("Unknown Item ({})", item_id))
    }

    /// Get full item info by ID
    pub fn get_item_info(&self, item_id: u32) -> Option<&ClientItemInfo> {
        self.items.get(&item_id)
    }

    /// Calculate total stat bonuses from equipped items
    pub fn calculate_equipment_bonuses(&self, equipment: &Equipment) -> ClientStatBonuses {
        let mut total = ClientStatBonuses::default();

        // Add bonuses from each equipped item
        if let Some(weapon_id) = equipment.weapon {
            if let Some(item) = self.items.get(&weapon_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(helmet_id) = equipment.helmet {
            if let Some(item) = self.items.get(&helmet_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(chest_id) = equipment.chest {
            if let Some(item) = self.items.get(&chest_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(legs_id) = equipment.legs {
            if let Some(item) = self.items.get(&legs_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(boots_id) = equipment.boots {
            if let Some(item) = self.items.get(&boots_id) {
                total.add(&item.stat_bonuses);
            }
        }

        total
    }
}

/// Client-side item information
#[derive(Clone, Debug)]
pub struct ClientItemInfo {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub stat_bonuses: ClientStatBonuses,
}

/// Client-side stat bonuses (mirrors server-side ItemStatBonuses)
#[derive(Clone, Default, Debug)]
pub struct ClientStatBonuses {
    pub attack_power: f32,
    pub defense: f32,
    pub max_health: f32,
    pub max_mana: f32,
    pub crit_chance: f32,
}

impl ClientStatBonuses {
    /// Add another set of bonuses to this one
    pub fn add(&mut self, other: &ClientStatBonuses) {
        self.attack_power += other.attack_power;
        self.defense += other.defense;
        self.max_health += other.max_health;
        self.max_mana += other.max_mana;
        self.crit_chance += other.crit_chance;
    }
}
