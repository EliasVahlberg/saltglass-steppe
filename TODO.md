# Saltglass Steppe - Development TODO

## Phase 1: Core Foundation ✓
- [x] Project setup (Cargo.toml, dependencies)
- [x] Basic TUI app loop (ratatui + crossterm)
- [x] Game state structure
- [x] Input handling system

## Phase 2: Map & Rendering ✓
- [x] Tile/map data structures
- [x] Map rendering to terminal
- [x] Player entity and movement
- [x] Collision detection

## Phase 3: Core Systems ✓
- [x] Turn-based game loop
- [x] FOV (field of view)
- [x] Message log
- [x] Deterministic RNG (seeded)

## Phase 4: Vertical Slice Content ✓
- [x] Procedural map generation (rooms + corridors)
- [x] Storm forecast UI
- [x] Map-edit storm event (walls → glass)
- [x] 3 enemy types (Mirage Hound, Glass Beetle, Salt Mummy)
- [x] Basic combat (bump-to-attack)
- [x] Enemy AI (pathfinding, chase player)
- [x] Death state

## Phase 5: Persistence & Progression ✓
- [x] Save/load system (RON format)
- [x] Refraction meter
- [x] 4 Adaptations (Prismhide, Sunveins, MirageStep, Saltblood)
- [x] Sharp glass damage on movement
- [x] Adaptation effects (Saltblood immunity, Sunveins +dmg)

## Phase 6: Items & Loot ✓
- [x] Item system (5 item types)
- [x] Item spawning in rooms
- [x] Auto-pickup on movement
- [x] Inventory display
- [x] Usable items (Brine Vial heals, Angle Lens reveals map)
- [x] Featured relic (Angle-Split Lens)

## Next Steps (Creative Direction Aligned)

### High Priority — Core Pillars
- [ ] Faction NPCs (Mirror Monks, Sand-Engineers, Glassborn) — *Pillar 1 requires NPCs who react to adaptations*
- [ ] Shimmer/glare visual effects (Pillar 3: Readable Light Tactics)
- [ ] Post-storm diff highlighting — changed tiles in LightCyan until visited (Pillar 2: Storms Rewrite Maps)

### Medium Priority — Content & Systems
- [ ] Beam/ray visualization using `-|/\` in Cyan (Pillar 3)
- [ ] Prismhide damage reduction
- [ ] MirageStep decoy mechanic
- [ ] More enemy behaviors (unique per type)
- [ ] Add remaining enemies: Archive Drone (`Δ`), Refraction Wraith (`◊`)
- [ ] Enhanced storm forecast panel (show edit types: ROTATE, GLASS, SWAP)

### Low Priority — Polish
- [ ] Extract color constants to `colors.rs` module
- [ ] Glare/hot light tiles (`░`)
- [ ] Storm shimmer overlay (`≈`) during active storms

## Tests: 13 passing
- deterministic_map_generation
- player_spawns_on_floor
- player_cannot_walk_through_walls
- storm_converts_walls_to_glass
- fov_includes_player_position
- enemies_spawn_in_rooms
- combat_reduces_enemy_hp
- save_load_roundtrip
- glass_increases_refraction
- saltblood_prevents_glass_damage
- items_spawn_in_map
- pickup_adds_to_inventory
- brine_vial_heals

---
Run: `cargo run` | Test: `cargo test`
