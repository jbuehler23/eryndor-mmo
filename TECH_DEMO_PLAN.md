# Eryndor MMO Tech Demo Launch Plan

## Overview

**Goal**: Launch a polished tech demo as a game development portfolio piece
**Audience**: Gaming community via closed alpha (invite-only Discord, 20-50 testers)
**Timeline**: Quality over speed - ship when ready (hobby project, no hard deadline)

### Core Design: "Build Your Role"

**Your role emerges from your skill choices and playstyle, not restrictions.**

Every layer provides options and trade-offs, but **nothing is locked**:

| Layer | Provides | Trade-offs |
|-------|----------|------------|
| **Class** | Starting stats + exclusive skills | Knight starts tankier, Mage starts with more mana |
| **Weapon** | Combat skills + damage style | Staff has heals, Sword is balanced, Dagger is fast |
| **Armor** | Stat bonuses + defensive style | Plate = defense, Cloth = mana, Leather = evasion |
| **Skills** | **YOUR ROLE** | Tank skills = Tank, Healer skills = Healer, DPS skills = DPS |

**The key insight:** Role = which skills you equip and how you play, NOT your gear.

**Example Builds (all viable):**
- Mage + Staff + Plate = **Battle Mage Tank** (magic + heavy armor)
- Mage + Staff + Plate = **Armored Healer** (heals from safety of plate)
- Mage + Wand + Cloth = **Glass Cannon DPS** (max spell damage)
- Knight + Mace + Chain = **Paladin Healer** (melee healer)
- Knight + Sword + Leather = **Evasion Fighter** (dodge + parry)
- Rogue + Dagger + Plate = **Armored Assassin** (burst + survivability)
- Rogue + Staff + Cloth = **Shadow Priest** (healer with stealth utility)

### Core Gameplay Loop

```
1. Kill enemies → Gain weapon/armor proficiency XP
2. Reach proficiency milestone (5, 10, 15, 20, 25, 30)
3. Talk to trainer → Accept skill quest
4. Complete quest → Learn new ability
5. Use new abilities → Kill harder enemies → Repeat
```

### Multiplayer Highlight

**Raid Boss** requiring 2-4 players with distinct roles (Tank + Healer + DPS)

---

## Phase 1: Critical Bug Fixes (~3 days)

### 1.1 Fix NPC Interaction Bug
**Root Cause**: `Interactable` component may not be replicated to client, making NPCs unclickable.

**Files to Modify**:
- `crates/eryndor_shared/src/lib.rs` - Ensure `Interactable` is registered for replication
- `crates/eryndor_client/src/input.rs` - Add debug logging, verify targeting query includes NPCs

**Testing**: Click Elder NPC → Quest dialogue opens → Accept quest works

### 1.2 Fix WeaponType Mapping
**Issue**: `weapon.rs` only maps Sword/Dagger/Wand, missing Staff/Mace/Bow/Axe.

**File**: `crates/eryndor_server/src/weapon.rs` (lines 75-82)

### 1.3 Verify Quest/Shop Flow
After NPC fix, test full flow:
- Quest acceptance with Elder NPC
- Shop purchases with Weapon Master NPC

---

## Phase 2: Core Solo Loop (~6 days)

### 2.1 Weapon Proficiency Gating
Gate better weapons behind proficiency milestones (10, 20, 30, 40, 50).

**Files**:
- `crates/eryndor_server/src/game_data.rs` - Add `required_proficiency: Option<(WeaponType, u32)>` to ItemDefinition
- `crates/eryndor_server/src/inventory.rs` - Check proficiency on equip

### 2.2 Dynamic XP Rewards
**Files**:
- `crates/eryndor_server/src/game_data.rs` - Add `experience_reward: u32` to EnemyDefinition
- `crates/eryndor_server/src/combat.rs` - Use enemy's XP reward instead of hardcoded 50

### 2.3 Enemy Loot Tables
Add meaningful drops to all 6 enemy types (currently only Goblin/Orc have item loot).

**Content to Create**:
- `assets/content/loot_tables/slime_loot.loot.json`
- `assets/content/loot_tables/wolf_loot.loot.json`
- `assets/content/loot_tables/skeleton_loot.loot.json`
- `assets/content/loot_tables/spider_loot.loot.json`

### 2.4 Quest Chain (Tutorial → Endgame)
**Quests to Create** (in `assets/content/quests/`):
1. "First Steps" (exists) - Kill 5 slimes
2. "Combat Training" - Equip weapon, use ability
3. "Weapon Mastery" - Reach proficiency level 5
4. "Exploring the Wilds" - Kill 3 wolves
5. "The Goblin Threat" - Kill 10 goblins
6. "Into the Graveyard" - Kill 5 skeletons
7. "Orc Stronghold" - Kill 3 orcs

---

## Phase 3: Progression & Role System (~7 days)

### 3.0 Core Design Philosophy: "Build Your Role"

**Any class can become any role** based on their proficiency choices:
- **Class** = Starting stats + class-specific advanced abilities
- **Weapon Proficiency** = Combat style + weapon techniques
- **Armor Proficiency** = Stat trade-offs (NOT role restrictions)

**Example Builds:**
| Class | Weapon | Armor | Resulting Role |
|-------|--------|-------|----------------|
| Knight | Sword | Plate | Classic Tank |
| Knight | Mace | Chain | Paladin (Healer) |
| Mage | Staff | Cloth | Pure Healer |
| Mage | Wand | Cloth | DPS Caster |
| Rogue | Dagger | Leather | Evasion Tank |
| Rogue | Bow | Leather | Ranged DPS |

---

### 3.1 Trainer System Architecture

**13 Trainers Total - ALL skills available to ANY class/build**

**Weapon Trainers (7)** - Combat skills anyone can learn:
| Trainer | Skills Include | Skill Types |
|---------|----------------|-------------|
| Sword Master | Cleave, Riposte, Whirlwind, Execute, Bladestorm | Damage, AoE, Counter |
| Dagger Master | Backstab, Flurry, Assassinate, Shadowstrike, Death Mark | Burst, DoT, Mobility |
| Staff Master | Heal Wave, Barrier, Restoration, Mass Heal, Divine Hymn | Heals, Shields, Support |
| Wand Master | Arcane Blast, Chain Lightning, Meteor, Arcane Barrage | Ranged, AoE, Burst |
| Mace Master | Stun, Judgment, Holy Strike, Consecrate, Hammer of Justice | CC, Hybrid Heal/Damage |
| Bow Master | Aimed Shot, Volley, Snipe, Rain of Arrows, Kill Shot | Ranged, AoE, Execute |
| Axe Master | Rend, Berserker Rage, Decapitate, Rampage, Bloodbath | Damage, Self-buff, Bleed |

**Armor Trainers (3)** - Defensive/utility skills anyone can learn:
| Trainer | Armor Types | Skill Types |
|---------|-------------|-------------|
| Cloth Weaver | Cloth | Mana management, Barriers, Utility |
| Leather Worker | Leather | Evasion, Mobility, Dodge |
| Armorer | Chain & Plate | Defense buffs, Damage reduction, Protection |

**Class Trainers (3)** - Class-exclusive flavor abilities:
| Trainer | Class | Unique Flavor |
|---------|-------|---------------|
| Knight Commander | Knight | Defensive stances, Shouts, Intervention |
| Archmage | Mage | Mana manipulation, Polymorphs, Time magic |
| Shadow Master | Rogue | Stealth, Poisons, Combo system |

**Key Design Principle:** Trainers teach skills, not roles. A Mage can learn Plate armor skills. A Knight can learn Staff heals. Your build is YOUR choice.

---

### 3.2 Quest-Based Skill Unlocks

**Unlock Flow:**
1. Player reaches proficiency milestone (5, 10, 15, 20, 25, 30)
2. Trainer offers quest: "Prove your mastery by [objective]"
3. Player completes quest (kill X enemies, use ability Y times, etc.)
4. Trainer teaches new ability

**Proficiency Milestones (every 5 levels):**
| Level | Quest Difficulty | Skill Tier |
|-------|------------------|------------|
| 5 | Easy (kill 10 mobs) | Basic |
| 10 | Medium (kill 25 mobs) | Intermediate |
| 15 | Medium (use ability 50 times) | Advanced |
| 20 | Hard (kill elite enemy) | Expert |
| 25 | Hard (complete dungeon) | Master |
| 30 | Very Hard (raid boss) | Grandmaster |

**Example Quest Chain (Sword Master):**
```
Sword Proficiency 5:  "Sword Basics" - Kill 10 enemies with sword → Learn: Cleave
Sword Proficiency 10: "Blade Discipline" - Kill 25 enemies → Learn: Riposte
Sword Proficiency 15: "Art of the Parry" - Parry 30 attacks → Learn: Deflection
Sword Proficiency 20: "Duelist's Challenge" - Defeat Goblin Champion → Learn: Whirlwind
Sword Proficiency 25: "Warrior's Trial" - Clear Skeleton Crypt → Learn: Execute
Sword Proficiency 30: "Blade Master" - Defeat Warchief Grok → Learn: Bladestorm
```

---

### 3.3 Armor Proficiency System

**Armor provides stat trade-offs - NOT role restrictions.**
Any class can wear any armor. Your role comes from your skill choices.

**Armor Types & Trade-offs:**

| Armor | Defense | Evasion | Mana | Speed | Best For |
|-------|---------|---------|------|-------|----------|
| Cloth | +0% | +0% | +50% | +10% | High mana pool, fast casting |
| Leather | +10% | +25% | +20% | +5% | Evasion builds, mobility |
| Chain | +25% | +10% | +10% | +0% | Balanced hybrid builds |
| Plate | +50% | -10% | -20% | -5% | Maximum survivability |

**Armor Trainer Skills (available to ALL who train, regardless of class/role):**

**Cloth Weaver** - Utility & mana management:
- Level 5: Meditation (channel to restore mana quickly)
- Level 10: Arcane Barrier (absorb shield on self/ally)
- Level 15: Spirit Link (share damage with ally)
- Level 20: Mana Burn (damage enemy mana, deal damage)
- Level 25: Phase Shift (brief intangibility, avoid damage)
- Level 30: Arcane Explosion (AoE burst centered on self)

**Leather Worker** - Mobility & evasion:
- Level 5: Quick Reflexes (passive dodge chance +)
- Level 10: Roll (short dash, avoid attacks)
- Level 15: Evasion (greatly increased dodge for 6s)
- Level 20: Riposte (counter-attack on successful dodge)
- Level 25: Blur (50% miss chance on attacks against you)
- Level 30: Ghost Walk (untargetable + move through enemies)

**Armorer - Chain** - Hybrid survivability:
- Level 5: Fortify (small defense buff)
- Level 10: Second Skin (reduce crit damage taken)
- Level 15: Aura of Protection (nearby allies +defense)
- Level 20: Divine Shield (immune to damage briefly)
- Level 25: Sacrifice (take damage meant for ally)
- Level 30: Guardian Spirit (prevent death once, heal to 20%)

**Armorer - Plate** - Heavy defense:
- Level 5: Shield Block (block next attack)
- Level 10: Armor Mastery (passive defense +)
- Level 15: Shield Wall (50% damage reduction, can't attack)
- Level 20: Last Stand (double current HP temporarily)
- Level 25: Reflective Armor (return % damage to attackers)
- Level 30: Unbreakable (immune to all damage for 5s)

---

### 3.4 Class Trainer Skills

**Knight Commander (Knight-Exclusive):**
- Level 5: Defensive Stance (threat+, damage-)
- Level 10: Shield Slam (stun + high threat)
- Level 15: Rally Cry (group buff)
- Level 20: Intervene (take hit for ally)
- Level 25: Bulwark (massive defense boost)
- Level 30: Avatar of War (all stats boosted)

**Archmage (Mage-Exclusive):**
- Level 5: Arcane Intellect (mana buff)
- Level 10: Mana Shield (damage to mana)
- Level 15: Counterspell (interrupt)
- Level 20: Polymorph (CC transform)
- Level 25: Time Warp (group haste)
- Level 30: Arcane Power (spell damage burst)

**Shadow Master (Rogue-Exclusive):**
- Level 5: Stealth (invisibility)
- Level 10: Cheap Shot (stun from stealth)
- Level 15: Kidney Shot (stun combo)
- Level 20: Cloak of Shadows (magic immune)
- Level 25: Vanish (drop combat + stealth)
- Level 30: Shadow Dance (abilities usable as if stealthed)

---

### 3.5 Tiered Gear System

**Tiers by Enemy Difficulty:**
| Tier | Source | Stat Bonus | Prof Req |
|------|--------|------------|----------|
| 1 | Slimes | Base | 0 |
| 2 | Wolves/Goblins | +15% | 10 |
| 3 | Skeletons/Spiders | +30% | 20 |
| 4 | Orcs/Boss | +50% | 30 |

**Gear Content to Create:**
- 7 weapon types x 4 tiers = 28 weapons
- 4 armor types x 4 tiers = 16 armor pieces

---

### 3.6 Implementation Files

**Extend Trainer Component:**
```rust
pub struct Trainer {
    pub items_for_sale: Vec<TrainerItem>,
    pub trainer_type: TrainerType,
    pub teaching_quests: Vec<u32>,  // NEW
}

pub enum TrainerType {
    Weapon(WeaponType),
    Armor(ArmorType),
    Class(CharacterClass),
}
```

**Files to Modify:**
- `crates/eryndor_shared/src/components.rs` - Extend Trainer, add ArmorProficiency tracking
- `crates/eryndor_server/src/game_data.rs` - Add trainer quest definitions
- `crates/eryndor_server/src/trainer.rs` - Handle quest-based ability teaching
- `crates/eryndor_server/src/quest.rs` - Add proficiency-gated quest acceptance

---

## Phase 4: Raid Boss (~4.5 days)

### 4.1 Create Boss Enemy
**"Warchief Grok"** - Orc boss requiring 2-4 players (impossible solo by design)

**Stats** (tuned for group requirement):
- Health: 3000 (20x normal Orc - too much for solo DPS)
- Attack: 50 (one-shots solo players without healer)
- Defense: 20
- Enrage Timer: 3 minutes (wipes raid if not killed)
- Uses abilities: Cleave (AoE), Ground Slam (must spread), War Cry (requires interrupt)
- Aggro/Leash Range: 300/600 (large)
- Loot: Guaranteed epic + 200-500 gold

**Why impossible solo**:
- High damage requires tank + healer coordination
- Enrage timer means solo DPS can't whittle it down
- Mechanics require multiple players to handle

**Files**:
- `assets/content/enemies/warchief_grok.enemy.json`
- `crates/eryndor_server/src/game_data.rs` - Add `is_boss`, `abilities` to EnemyDefinition
- `crates/eryndor_server/src/combat.rs` - Boss ability usage logic

### 4.2 Threat/Aggro System
Proper threat table so boss doesn't just focus one player.

**Files**:
- `crates/eryndor_shared/src/components.rs` - Add `ThreatTable` component
- `crates/eryndor_server/src/combat.rs` - Accumulate threat, target highest

### 4.3 Boss Spawn Location
Add boss spawn region to far corner of starter zone.

**File**: `assets/content/zones/starter_zone.zone.json`

### 4.4 Group Loot Distribution
Split gold, roll for items among nearby players who participated.

**File**: `crates/eryndor_server/src/inventory.rs`

---

## Phase 5: Polish & Onboarding (~5.5 days)

### 5.1 Visual Target Feedback
- Highlight on hover for clickable entities
- Selection indicator around targeted entity
- Range indicator (green = in range, red = too far)

**Files**:
- `crates/eryndor_client/src/input.rs`
- `crates/eryndor_client/src/rendering.rs`

### 5.2 Tutorial/Onboarding
- First-login tutorial tooltips
- UI indicators for NPCs, combat, abilities
- Contextual hints

**File**: `crates/eryndor_client/src/ui.rs`

### 5.3 Combat Log
Show damage dealt/received in dedicated panel.

**File**: `crates/eryndor_client/src/ui.rs`

### 5.4 Hotbar Improvements
- Visual cooldown indicators
- Ability icons (colored squares with symbols)

### 5.5 Death/Respawn Polish
- Death fade out animation
- Respawn at safe location
- Brief invulnerability

**Files**:
- `crates/eryndor_server/src/combat.rs`
- `crates/eryndor_client/src/rendering.rs`

---

## Phase 6: Deployment & Launch (~3 days)

### 6.1 Configure GitHub Secrets
Add to GitHub repo settings:
- `DO_API_TOKEN` - DigitalOcean API token
- `DO_SSH_PRIVATE_KEY` - SSH key for droplet
- `DO_DROPLET_IP` - 165.227.217.144
- `JWT_SECRET` - From .env.production

### 6.2 Create deploy-client.yml
Missing workflow for client deployment to DigitalOcean App Platform.

### 6.3 Performance Testing
Load test with 10-20 concurrent players.

### 6.4 Landing Page
Simple page with:
- "Play Now" button
- Controls reference
- Discord link for feedback

### 6.5 Launch Materials
- Reddit/Discord announcement
- Screenshots/GIFs
- Known issues list

---

## Post-Demo Roadmap

| Timeline | Feature | Description |
|----------|---------|-------------|
| Week 6-7 | Class Specialization | 2 specs per class, 3-4 unique abilities each |
| Week 8-9 | PvP Arena | Designated PvP zone, leaderboard |
| Week 10-11 | Crafting | Gather resources, craft at NPCs |
| Week 12+ | Second Zone | New area, zone transitions |
| Future | Party System | Formal grouping with shared quests |
| Future | Chat System | Global, local, whisper channels |
| Future | Editor Completion | Full world editor for content creation |

---

## Content Creation Summary

### NPCs to Create (13 Trainers)

**Weapon Trainers (7)**:
- `sword_master.npc.json` - Sword skills
- `dagger_master.npc.json` - Dagger skills
- `staff_master.npc.json` - Staff skills (healer focus)
- `wand_master.npc.json` - Wand skills (caster DPS)
- `mace_master.npc.json` - Mace skills (tank/healer)
- `bow_master.npc.json` - Bow skills (ranged DPS)
- `axe_master.npc.json` - Axe skills (melee DPS)

**Armor Trainers (3)**:
- `cloth_weaver.npc.json` - Cloth armor skills
- `leather_worker.npc.json` - Leather armor skills
- `armorer.npc.json` - Chain/Plate armor skills

**Class Trainers (3)**:
- `knight_commander.npc.json` - Knight-exclusive skills
- `archmage.npc.json` - Mage-exclusive skills
- `shadow_master.npc.json` - Rogue-exclusive skills

---

### Quests to Create (~78 Training Quests)

**Per Trainer: 6 quests (proficiency 5, 10, 15, 20, 25, 30)**
- 7 weapon trainers x 6 = 42 weapon quests
- 3 armor trainers x 6 = 18 armor quests (x2 for chain/plate split = 24)
- 3 class trainers x 6 = 18 class quests

**Story/Tutorial Quests (7)**:
- `first_steps.quest.json` (exists)
- `combat_training.quest.json`
- `choose_your_weapon.quest.json`
- `armor_basics.quest.json`
- `exploring_wilds.quest.json`
- `goblin_threat.quest.json`
- `orc_stronghold.quest.json`

---

### Abilities to Create (~84 New Abilities)

**Weapon Skills (7 trainers x 6 skills = 42)**:
- Sword: Cleave, Riposte, Deflection, Whirlwind, Execute, Bladestorm
- Dagger: Stab, Backstab, Flurry, Assassinate, Shadowstrike, Death Mark
- Staff: Channel, Heal Wave, Barrier, Restoration, Mass Heal, Divine Hymn
- Wand: Bolt, Arcane Blast, Chain Lightning, Meteor, Arcane Barrage, Supernova
- Mace: Smash, Stun, Judgment, Holy Strike, Consecrate, Hammer of Justice
- Bow: Shoot, Aimed Shot, Volley, Snipe, Rain of Arrows, Kill Shot
- Axe: Chop, Rend, Berserker Rage, Decapitate, Rampage, Bloodbath

**Armor Skills (4 types x 6 skills = 24)**:
- Cloth: Meditation, Arcane Barrier, Spirit Link, Mana Burn, Phase Shift, Arcane Explosion
- Leather: Quick Reflexes, Roll, Evasion, Riposte, Blur, Ghost Walk
- Chain: Fortify, Second Skin, Aura of Protection, Divine Shield, Sacrifice, Guardian Spirit
- Plate: Shield Block, Armor Mastery, Shield Wall, Last Stand, Reflective Armor, Unbreakable

**Class Skills (3 classes x 6 skills = 18)**:
- Knight: Defensive Stance, Shield Slam, Rally Cry, Intervene, Bulwark, Avatar of War
- Mage: Arcane Intellect, Mana Shield, Counterspell, Polymorph, Time Warp, Arcane Power
- Rogue: Stealth, Cheap Shot, Kidney Shot, Cloak of Shadows, Vanish, Shadow Dance

---

### Items to Create

**Weapons (7 types x 4 tiers = 28)**:
- Format: `{weapon}_{tier}.item.json` (basic, improved, superior, epic)

**Armor (4 types x 4 tiers = 16)**:
- `cloth_{tier}.item.json`
- `leather_{tier}.item.json`
- `chain_{tier}.item.json`
- `plate_{tier}.item.json`

**Enemies (1)**:
- `warchief_grok.enemy.json`

**Loot Tables (4)**:
- `slime_loot.loot.json`
- `wolf_loot.loot.json`
- `skeleton_loot.loot.json`
- `spider_loot.loot.json`

---

## Critical Files Reference

| File | Purpose |
|------|---------|
| `crates/eryndor_client/src/input.rs` | NPC targeting bug, click detection |
| `crates/eryndor_server/src/combat.rs` | Combat loop, boss abilities, threat |
| `crates/eryndor_server/src/game_data.rs` | Item/enemy definitions, requirements |
| `crates/eryndor_shared/src/components.rs` | New components (ThreatTable) |
| `crates/eryndor_shared/src/lib.rs` | Replication registration |
| `crates/eryndor_server/src/inventory.rs` | Equip requirements, loot distribution |
| `crates/eryndor_server/src/weapon.rs` | WeaponType mapping fix |
| `assets/content/zones/starter_zone.zone.json` | Zone layout, boss spawn |

---

## Timeline Summary

| Phase | Duration |
|-------|----------|
| Bug Fixes | 3 days |
| Solo Loop | 6 days |
| Progression & Role System | 7 days |
| Raid Boss | 4.5 days |
| Polish | 5.5 days |
| Launch | 3 days |
| **Total** | **~29 days (~6 weeks)** |

**Note:** Content creation (84 abilities, 78 quests, 13 NPCs) can be parallelized or done incrementally. Start with 1 trainer per category for MVP, expand post-launch.

---

## Player Acquisition Strategy

**Approach**: Closed Alpha via Discord

### Setup
1. Create Discord server for Eryndor (if not exists)
2. Set up channels: #announcements, #feedback, #bug-reports, #general
3. Create invite link with limited uses (50)

### Recruitment Sources
- r/indiegaming, r/playmygame - "Looking for alpha testers" posts
- Bevy Discord - fellow Rust gamedevs interested in testing
- Personal network - friends/colleagues who game
- GameDev forums - other indie devs for feedback exchange

### Feedback Collection
- In-game: F12 opens feedback form (sends to Discord webhook)
- Discord: Structured feedback template in #feedback
- Post-session surveys (Google Form)

### Portfolio Presentation
Since this is a portfolio piece, document the development:
- Dev blog/Twitter thread showing progress
- GitHub README with architecture overview
- Record gameplay video for portfolio
- Write technical post about bevy_replicon multiplayer architecture
