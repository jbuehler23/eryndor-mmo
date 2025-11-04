# Eryndor MMO - TODO List

## Critical Bugs

### 🔴 NPC Interaction Not Working
**Priority: HIGH**

**Problem:**
- Players cannot target NPCs by clicking on them
- No targeting logs appear when clicking NPCs
- Quest system is inaccessible

**Root Cause:**
- The targeting system in `input.rs:handle_targeting_input()` queries for `Or<(With<Enemy>, With<Npc>, With<WorldItem>)>`
- NPCs may not be getting targeted properly - click detection radius or query might be failing

**Proposed Solution:**
1. Add an `Interactable` component to mark all clickable entities:
   ```rust
   #[derive(Component, Reflect)]
   pub struct Interactable {
       pub interaction_radius: f32,
       pub interaction_type: InteractionType,
   }

   pub enum InteractionType {
       NpcDialogue,
       ItemPickup,
       Harvest,
       Door,
       LoreObject,
   }
   ```

2. Update targeting system to query for `With<Interactable>` instead
3. Add visual feedback (highlight/outline) when hovering over interactable entities
4. Add interaction radius visualization (optional debug feature)

**Files to Modify:**
- `crates/eryndor_shared/src/components.rs` - Add Interactable component
- `crates/eryndor_server/src/world.rs` - Add Interactable to NPCs, items, etc.
- `crates/eryndor_client/src/input.rs` - Update targeting query
- `crates/eryndor_client/src/main.rs` - Register Interactable for replication

**Testing:**
1. Spawn NPC with Interactable component
2. Click on NPC - verify "Selected target" log appears
3. Press E - verify "Interacting with NPC" log and quest dialogue opens
4. Test with items, enemies, etc.

---

## High Priority Features

### 🟡 Visual Feedback for Targeting
**Priority: MEDIUM**

Add visual indicator when entity is:
- Hoverable (cursor changes, subtle highlight)
- Selected (border/outline around entity)
- In interaction range (green indicator)
- Out of interaction range (red indicator)

**Implementation:**
- Use bevy_prototype_lyon stroke/outline
- Add TargetIndicator component
- Spawn/update indicator entity in rendering system

---

### 🟡 Interaction Range Validation
**Priority: MEDIUM**

Currently E works regardless of distance to target.

**Add:**
1. Server-side range check in interaction handlers
2. Client-side pre-check to show "Too far away" message
3. Configurable interaction range per entity type

---

## Medium Priority Features

### 🟢 Multiple Interactable Types
**Priority: LOW-MEDIUM**

Expand interaction system beyond NPCs:
- [ ] Harvest nodes (trees, ore, plants)
- [ ] Doors/portals
- [ ] Lore objects (books, signs, monuments)
- [ ] Containers (chests, barrels)
- [ ] Crafting stations

---

### 🟢 UI Improvements
**Priority: MEDIUM**

Current UI issues:
- [ ] Inventory grid layout could be better
- [ ] No tooltips on hover
- [ ] Quest log needs better formatting
- [ ] Combat log/damage numbers
- [ ] Target frame shows entity ID instead of name

---

### 🟢 Camera Improvements
**Priority: LOW**

- [ ] Smooth camera lerp (currently instant follow)
- [ ] Camera zoom controls (mouse wheel)
- [ ] Camera bounds (don't show outside world)

---

## Low Priority / Polish

### 🔵 Quality of Life
- [ ] Double-tap WASD for dodge/dash
- [ ] Tab key cycles through nearby enemies
- [ ] Right-click for quick interactions
- [ ] Keybind customization
- [ ] Settings menu (graphics, audio, controls)

### 🔵 Audio
- [ ] Background music
- [ ] Ambient sounds
- [ ] Combat sound effects
- [ ] UI click sounds
- [ ] Footstep sounds

### 🔵 Visual Polish
- [ ] Sprite sheets instead of shapes
- [ ] Particle effects (abilities, damage, healing)
- [ ] Screen shake on impactful events
- [ ] Minimap
- [ ] Health bar above entities

---

## Technical Debt

### ⚙️ Code Quality
- [ ] Remove duplicate "Menu" window names (causing egui ID conflicts) ✅ DONE
- [ ] Refactor input systems to be more modular
- [ ] Add error handling for network failures
- [ ] Implement proper logging levels (trace, debug, info, warn, error)
- [ ] Add unit tests for game logic
- [ ] Add integration tests for client-server communication

### ⚙️ Performance
- [ ] Profile and optimize entity queries
- [ ] Implement spatial partitioning for large worlds
- [ ] Add LOD system for distant entities
- [ ] Optimize replication (only replicate relevant entities per client)

### ⚙️ Security
- [ ] Add rate limiting for client requests
- [ ] Implement anti-cheat measures
- [ ] Sanitize user input (character names, chat messages)
- [ ] Add session management (prevent duplicate logins) ✅ DONE
- [ ] Encrypt sensitive data in database

---

## Future Systems (Post-MVP)

### 🌟 Combat Enhancements
- [ ] More abilities per class
- [ ] Skill trees
- [ ] Status effects (buffs/debuffs)
- [ ] Area of effect abilities
- [ ] Combo system

### 🌟 Social Features
- [ ] Chat system (global, local, whisper)
- [ ] Party system
- [ ] Guild system
- [ ] Friend list
- [ ] Trade system

### 🌟 World Features
- [ ] Multiple zones/maps
- [ ] Zone transitions
- [ ] Dynamic events
- [ ] Weather system
- [ ] Day/night cycle

### 🌟 Economy
- [ ] Currency system
- [ ] Shop NPCs
- [ ] Auction house
- [ ] Crafting system
- [ ] Gathering professions

### 🌟 Character Progression
- [ ] Leveling system (beyond level 1)
- [ ] Stat allocation
- [ ] Equipment system (armor, accessories)
- [ ] Character customization (appearance)

---

## Completed Recently

- ✅ Character name labels above players
- ✅ ESC menu for character disconnect/switching
- ✅ Visual entity cleanup on disconnect
- ✅ Fixed duplicate window ID error (Menu/Actions)
- ✅ Input blocking when ESC menu is open
- ✅ Database persistence on character disconnect
- ✅ Camera following player
- ✅ Prevent duplicate account logins

---

## Notes

### Interaction System Design Philosophy

The new `Interactable` component approach is more extensible than hardcoding specific entity types (Npc, WorldItem, etc.) because:

1. **Flexibility**: Any entity can become interactable by adding the component
2. **Type Safety**: InteractionType enum prevents bugs
3. **Configurability**: Different interaction ranges per entity
4. **Future-Proof**: Easy to add new interaction types (doors, harvest nodes, etc.)
5. **Visual Feedback**: Can query all interactables for highlighting

### Development Workflow

1. Fix critical bugs first (NPC interaction)
2. Add visual feedback (targeting indicators)
3. Polish existing features (UI, camera)
4. Add new systems (audio, more content)
5. Technical debt and optimization

### Testing Multiplayer

Always test with at least 2 clients:
```bash
# Terminal 1
cargo run -p eryndor_server

# Terminal 2
cargo run -p eryndor_client

# Terminal 3
cargo run -p eryndor_client
```

Create different accounts and verify:
- Both clients see each other
- Name labels appear correctly
- Disconnecting one client removes their character from the other's view
- Character switching works without restart
