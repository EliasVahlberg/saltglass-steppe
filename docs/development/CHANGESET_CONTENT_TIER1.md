# Changeset: Tier 1 Content — Saltglass Steppe

> For: LeadDeveloper
> From: Creative Director
> Date: 2026-05-02
> Type: Content (no new systems)
> Scope: Item/weapon/chest/skill renames, placeholder quest rewrites, missing quest items, tonal-break event rewrites
> Rule: All changes are to existing JSON data fields. No Rust code changes. No new schemas.

---

## 1. Item Renames (items.json)

Rename generic items to use the world's material vocabulary.

| Old ID | Old Name | New ID | New Name | New Description |
|--------|----------|--------|----------|----------------|
| `leather_jacket` | Leather Jacket | `brine_cured_hide` | Brine-Cured Hide | Steppe leather soaked in brine until the salt crystallized into the grain. Stiff, heavy, and smells like low tide. Stops a glass shard better than it stops the cold. |
| `reinforced_jacket` | Reinforced Jacket | `salt_hardened_hide` | Salt-Hardened Hide | Hide jacket with salt-crystal plates sewn between the layers. The plates crack on impact and reform overnight if you leave it in the damp. Engineers call it self-healing armor. Monks call it alive. |
| `hand_torch` | Hand Torch | `fulgurite_brand` | Fulgurite Brand | Oiled rag wrapped around a fulgurite shard. The glass holds heat longer than wood and the light has a blue-white edge that makes shadows behave strangely. |
| `dried_meat` | Dried Meat | `salt_cured_rat` | Salt-Cured Rat | Rat soaked in brine for three days, then dried in the sun. Chewy, salty, and keeps for months. Don't ask what kind of rat. |
| `preserved_rations` | Preserved Rations | `hermit_rations` | Hermit Rations | A Salt Hermit's travel pack: brine-crust bread, dried kelp, and a twist of cactus needles for tea. Tastes like survival. |
| `water_flask` | Water Flask | `brine_flask` | Brine Flask | Sealed flask of filtered brine. Not delicious, but it keeps you walking. The salt residue on the cap tells you how old it is. |
| `rope` | Rope | `glass_fiber_rope` | Glass-Fiber Rope | Braided from wire-grass and thin glass filaments. Stronger than hemp, lighter than chain, and it cuts your hands if you grip it wrong. |
| `cloth_scrap` | Cloth Scrap | `salt_rag` | Salt Rag | Strip of fabric stiff with salt. Useful for bandaging, filtering, or wrapping your boots when the flats start eating the leather. |

## 2. Weapon Renames (weapons.json)

| Old ID | Old Name | New ID | New Name | New Description |
|--------|----------|--------|----------|----------------|
| `rusty_blade` | Rusty Blade | `corroded_salt_blade` | Corroded Salt-Blade | Pre-storm steel eaten by decades of salt air. Still sharp where the corrosion hasn't reached. The pitting makes wounds that don't close clean. |
| `iron_hammer` | Iron Hammer | `slag_hammer` | Slag Hammer | Foundry hammer from before White Noon, head fused with glass slag during the flash. Heavier than it should be. The glass veins glow faintly after a storm. |
| `sling` | Sling | `salt_sling` | Salt Sling | Braided wire-grass sling that hurls crystallized salt chunks. The crystals shatter on impact, embedding shards. Cheap, quiet, and the ammunition is literally everywhere. |
| `throwing_knives` | Throwing Knives | `glass_throwing_shards` | Glass Throwing Shards | Knapped storm-glass fragments balanced for throwing. They catch the light in flight, which is beautiful and also gives away your position. |
| `pilgrim_staff` | Pilgrim Staff | `angle_staff` | Angle Staff | Walking staff with a glass-tipped head that refracts light at odd angles. Mirror Monks use them to read terrain. Everyone else uses them to not fall down. |
| `crystal_mace` | Crystal Mace | `fulgurite_mace` | Fulgurite Mace | Heavy bludgeon with a head of fused fulgurite — branching glass tubes that crackle with residual storm energy. Each hit sounds like breaking a window. |

## 3. Chest Renames (chests.json)

| Old ID | Old Name | New ID | New Name | New Description |
|--------|----------|--------|----------|----------------|
| `wooden_chest` | Wooden Chest | `salt_warped_crate` | Salt-Warped Crate | Pre-storm storage crate swollen and bleached by decades of salt exposure. The wood has crystallized at the grain. Pry it open carefully — the lid splinters into needles. |
| `metal_strongbox` | Metal Strongbox | `brine_sealed_strongbox` | Brine-Sealed Strongbox | Steel box with a brine-corroded lock. The salt crust around the seal means it hasn't been opened since before the storms changed the landscape around it. |
| `supply_crate` | Supply Crate | `engineer_supply_cache` | Engineer Supply Cache | Sand-Engineer field cache marked with their guild stamp. Standardized contents, standardized placement, standardized disappointment when someone else got there first. |

Note: `metal_key` (required by metal_strongbox) doesn't exist in items.json. Either add a `brine_key` item or change the strongbox to require `glass_pick` to force open.

## 4. Skill Renames (skill_trees.json)

Name changes only. No mechanical changes.

| Old Name | New Name | Category |
|----------|----------|----------|
| Bartering | Salt-Scrip Haggling | social |
| Scavenging | Glass-Field Scavenging | survival |
| Expert Scavenging | Deep Salvage | survival |
| Track & Trace | Salt-Sign Reading | survival |
| Basic Medical Practice | Salt-Wound Treatment | medical |
| Wound Packing | Shard Extraction | medical |
| Draw a Bead | Glare Compensation | ranged_combat |
| Sniper's Eye | Long-Glass Eye | ranged_combat |
| Weapon Modding | Storm-Tempering | crafting |
| Ammo Fabrication | Shard Pressing | crafting |

## 5. Placeholder Quest Rewrites (quests.json)

Replace 4 generic quests with world-grounded alternatives. Same mechanical structure (objective types, reward amounts), new IDs, names, descriptions, and flavor.

### 5.1 pest_control → glass_bloom_containment

```json
{
  "id": "glass_bloom_containment",
  "name": "Glass Bloom Containment",
  "description": "A shard spider nest is growing near the settlement's water source. If the bloom reaches the brine line, the spiders will vitrify the pipes. Clear the nest before the next storm feeds it.",
  "objectives": [
    { "type": "kill", "enemy_id": "shard_spider", "count": 3 },
    { "type": "reach", "description": "Inspect the brine line for damage" }
  ],
  "rewards": { "xp": 50, "salt_scrip": 100 },
  "repeatable": true
}
```

### 5.2 supply_run → brine_run

```json
{
  "id": "brine_run",
  "name": "Brine Run",
  "description": "The south brine line is dry. Someone needs to haul salt crystals from the nearest deposit before the pumps seize. The route crosses open flats — glare country. Bring goggles.",
  "objectives": [
    { "type": "collect", "item_id": "salt_crystal", "count": 3 },
    { "type": "reach", "description": "Deliver to the pump house" }
  ],
  "rewards": { "xp": 50, "salt_scrip": 75 },
  "repeatable": true
}
```

### 5.3 scout_mission → storm_scar_survey

```json
{
  "id": "storm_scar_survey",
  "name": "Storm Scar Survey",
  "description": "Last night's storm carved a new scar through the eastern flats. The Engineers need someone to walk it and report what changed — new corridors, collapsed routes, anything the maps need to forget.",
  "objectives": [
    { "type": "reach", "description": "Survey the storm scar" },
    { "type": "reach", "description": "Return with the survey" }
  ],
  "rewards": { "xp": 75, "salt_scrip": 100 },
  "repeatable": true
}
```

### 5.4 meet_merchant → the_glint_fence

```json
{
  "id": "the_glint_fence",
  "name": "The Glint Fence",
  "description": "There's a trader in Last Salt who deals in things that fell off the back of a caravan. Find the glint fence and see what they're selling. Don't mention where you heard about them.",
  "objectives": [
    { "type": "talk_to", "npc_id": "scavenger_trader" }
  ],
  "rewards": { "xp": 25, "salt_scrip": 50 },
  "repeatable": false
}
```

## 6. Tonal-Break Event Rewrites (dynamic_events.json)

Replace 2 slapstick events with world-grounded absurdity.

### 6.1 bureaucratic_confusion → archive_protocol_error

The humor should come from the Archive's inhuman logic, not fourth-wall breaking.

```json
{
  "id": "archive_protocol_error",
  "name": "Archive Protocol Error",
  "description": "A half-buried drone rises from the sand and addresses you by a name that isn't yours. It insists you are 217 days overdue for a scheduled maintenance appointment and attempts to escort you to a facility that no longer exists. It is very polite about this.",
  "trigger": { "biome": ["ruins", "shattered_citadel"], "chance": 0.08 },
  "outcomes": [
    { "choice": "comply", "text": "You follow the drone for twenty minutes before it stops, recalculates, and announces that your appointment has been rescheduled to a date that hasn't happened yet. It thanks you for your patience and sinks back into the sand.", "effect": {} },
    { "choice": "refuse", "text": "The drone notes your non-compliance, assigns you a demerit that will appear on a performance review no one will ever read, and returns to its patrol. You feel oddly guilty.", "effect": {} }
  ]
}
```

### 6.2 interpretive_disagreement → storm_reading_dispute

The humor should come from faction absurdity, not cosmic slapstick.

```json
{
  "id": "storm_reading_dispute",
  "name": "Storm Reading Dispute",
  "description": "A Mirror Monk and a Sand-Engineer are arguing over a storm scar. The Monk insists the pattern is a verse from the Litany of Refraction. The Engineer insists it's a pressure map showing aquifer depth. They both want you to settle it.",
  "trigger": { "biome": ["saltflat", "storm_scars"], "chance": 0.06 },
  "outcomes": [
    { "choice": "side_monk", "text": "You squint at the scar and say it looks like scripture. The Monk beams. The Engineer mutters something about confirmation bias and walks away. You're not sure you were right, but the Monk gives you a blessing that makes your compass needle twitch.", "effect": { "reputation_change": { "mirror_monks": 3, "sand_engineers": -2 } } },
    { "choice": "side_engineer", "text": "You point out the gradient lines and say it's clearly geological. The Engineer nods. The Monk says you have no poetry in your soul and leaves. The Engineer marks the aquifer on your map.", "effect": { "reputation_change": { "sand_engineers": 3, "mirror_monks": -2 } } },
    { "choice": "both_right", "text": "You suggest it could be both — a verse about water. They stare at you. Then at each other. Then they walk away together, arguing about whether that makes you a Synthesis Seeker or just indecisive.", "effect": { "reputation_change": { "synthesis_seekers": 2 } } }
  ]
}
```

## 7. Missing Quest Items (items.json)

These items are referenced in main_questline.json but don't exist in items.json. Add them.

| ID | Name | Tier | Type | Description |
|----|------|------|------|-------------|
| `broken_saint_key` | Broken Saint-Key | 2 | quest | A damaged saint-key with a partial credential imprint. It opens some Archive doors but flags you with incomplete credentials. Drones don't kill you immediately — they shadow and escalate. |
| `sacred_angle_lens` | Sacred Angle Lens | 3 | quest | A liturgical lens that completes a credential prayer when aligned with three Scripture Shards. The Mirror Monks consider it a temporary override — borrowed authority, not earned. |
| `forecast_instrument` | Storm Forecast Instrument | 3 | tool | Salvaged optics and power couplers assembled into a device that previews storm edit types before they arrive. The Engineers' answer to prophecy: prediction through measurement. |
| `spoof_module` | Credential Spoof Module | 3 | tool | A device that makes a broken saint-key behave like a valid one. Sometimes. The Engineers built it, the Monks call it blasphemy, and the drones call it a 73% authentication match. |
| `prime_lens_shard` | Prime Lens Shard | 4 | quest | A fragment of the Heliograph's master focusing element. It hums at a frequency that makes your teeth ache and your adaptations flare. Three of these exist. |
| `shard_of_clarity` | Shard of Clarity | 5 | quest | The knowledge aspect of the Prime Lens. Carries the Heliograph's targeting data — the precise understanding of what the correction loop was meant to achieve. Found in the Deep Archive Wing. |
| `shard_of_will` | Shard of Will | 5 | quest | The force aspect of the Prime Lens. Carries the Heliograph's power-channeling capacity. Hidden in the magma-glass caverns. Holding it feels like gripping a thunderbolt that hasn't decided whether to strike. |
| `shard_of_soul` | Shard of Soul | 5 | quest | The identity aspect of the Prime Lens. Carries the Heliograph's calibration matrix. Cannot be found in a ruin — must be earned through faction alignment. Responds to the bearer's choices. |
| `prime_lens_complete` | The Prime Lens | 6 | quest | The reassembled master focusing element of the Heliograph array. Three shards made whole. It doesn't glow — it *clarifies*. Everything around it looks more real than it did a moment ago. |

---

## Validation

After applying all changes:
- [ ] No broken ID references (old IDs removed from all cross-references)
- [ ] All quest objective item_ids and npc_ids exist in their respective files
- [ ] `cargo test` passes
