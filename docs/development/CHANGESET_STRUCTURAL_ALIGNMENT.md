# Changeset Spec: Structural Alignment (saltglass-steppe)

> For: LeadDeveloper (items 1–3), Creative Director (items 4–5 in follow-up)
> Date: 2026-05-01
> Scope: Faction unification, NPC dedup, new factions, locations schema
> Rule: This changeset is STRUCTURAL ONLY. No quest rewrites, no new recipes, no description changes. Those follow separately.

---

## 1. Faction ID Unification

### 1.1 Convention

**snake_case everywhere.** All faction IDs become snake_case in all files and all Rust source. This is consistent with every other ID type in the codebase (NPC IDs, item IDs, biome IDs, structure IDs).

### 1.2 Rename Map — Existing 7 Factions

| Old ID (PascalCase) | New ID (snake_case) | Display Name | Notes |
|---------------------|---------------------|-------------|-------|
| `MirrorMonks` | `mirror_monks` | Mirror Monks | |
| `SandEngineers` | `sand_engineers` | Sand-Engineers | |
| `Glassborn` | `glassborn` | Glassborn | Already lowercase, just standardize |
| `ArchiveDrones` | `archive_consciousness` | Archive Consciousness | Rename to match lore graph. "Drones" are the faction's agents, not the faction itself |
| `SaltTraders` | `salt_traders` | Salt Traders | Keep as minor faction |
| `StormCults` | `glass_prophets` | Glass Prophets | **Merge.** Storm Cults are the street-level expression of Glass Prophet ideology. See §1.5 |
| `RefractionOutcasts` | `refraction_outcasts` | Refraction Outcasts | Keep as minor faction |

### 1.3 New Factions to Add

Add these 4 to `factions.json`. They already exist in `npcs.json` (PascalCase) and the lore graph.

| ID | Display Name | Color | Description |
|----|-------------|-------|-------------|
| `synthesis_seekers` | Synthesis Seekers | White | Neutral mediators who believe science and spirituality are two faces of the same truth. Founded by Saint Matthias after the Schism Wars. They brokered the prisoner exchange that freed Brother Halix and maintain embassies on the Nexus Plateau. |
| `iron_covenant` | Iron Covenant | DarkGray | Anti-adaptation militants who believe all transformation is contamination. Led by Forge-Master Kaine Durgan. They develop null-field weapons and hunt heavily adapted individuals. Pragmatic allies of the Sand-Engineers when it suits them. |
| `glass_prophets` | Glass Prophets | Red | Radical accelerationists who believe the final storm will complete humanity's transformation into beings of pure light. Led by The Prism. Born from the Schism Wars, they seek to trigger massive storm events and establish ascension sites. Absorbs the former Storm Cults. |
| `wandering_court` | Wandering Court | DarkMagenta | Moderate Glassborn who preserve human culture while embracing gradual transformation. Led by Echo-of-Memory. They provide guidance to newly adapted individuals and serve as intermediaries between the Glassborn collective and baseline humanity. |

### 1.4 Phantom Faction Mapping

These IDs appear in data files but are not real factions. Map them:

| Phantom ID | Maps To | Rationale | Files Affected |
|-----------|---------|-----------|----------------|
| `saltborn_collective` | `salt_traders` | Same faction, different name | dialogues.json, traders.json, npcs.json |
| `glasswright_guild` | `glassborn` | Guild within the Glassborn collective | dialogues.json, traders.json, recipes.json |
| `storm_prophets` | `glass_prophets` | Same faction, different name | dialogues.json, traders.json |
| `IronCovenant` | `iron_covenant` | Case change only | npcs.json |
| `GlassProphets` | `glass_prophets` | Case change only | npcs.json |
| `WanderingCourt` | `wandering_court` | Case change only | npcs.json |
| `SynthesisSeekers` | `synthesis_seekers` | Case change only | npcs.json |
| `HeliographNetwork` | `archive_consciousness` | Heliograph is part of the Archive system | npcs.json |
| `ArchiveConsciousness` | `archive_consciousness` | Case change only | npcs.json |
| `ARIA` | `archive_consciousness` | ARIA is the Archive's AI personality | npcs.json |
| `Archive` | `archive_consciousness` | Shorthand for the same faction | npcs.json |
| `independent` | `independent` | Keep as-is. Special tag for unaffiliated traders | traders.json |
| `hermits` | — | **Not a faction.** Remove from narrative_integration.json faction_influences. Keep as NPC description tag only | narrative_integration.json |
| `GlassbornCollective` | `glassborn` | Verbose form of the same faction | npcs.json |
| `VoidWardens` | — | NPC flavor tag, not a faction. Keep in NPC description, remove from faction field | npcs.json |
| `LightKeepers` | — | NPC flavor tag, not a faction | npcs.json |
| `StormMemorialists` | — | NPC flavor tag, not a faction | npcs.json |
| `CosmicWatch` | — | NPC flavor tag, not a faction | npcs.json |
| `QuantumCollective` | — | NPC flavor tag, not a faction | npcs.json |

For NPCs with flavor-tag factions (VoidWardens, LightKeepers, etc.): set their `faction` field to the nearest real faction or `independent`, and move the flavor tag to their `description`.

### 1.5 StormCults → glass_prophets Merge

The Storm Cults and Glass Prophets share the same ideology (pro-transformation extremism). The distinction is organizational: Glass Prophets are the leadership, Storm Cults are the street-level followers. Merge them under `glass_prophets`.

**Structures to rename:**
| Old Structure ID | New Structure ID |
|-----------------|-----------------|
| `storm_shrine` | `glass_prophets_shrine` |
| `ritual_circle` | `glass_prophets_ritual_circle` |
| `tempest_observatory` | `glass_prophets_observatory` |

Update `metadata.faction` from `storm_cults` to `glass_prophets` in these structures.
Update `metadata.tags` from `["faction", "storm_cults"]` to `["faction", "glass_prophets"]`.

**NPCs to re-faction:**
- `storm_priest` → faction: `glass_prophets`
- `storm_cultist` → faction: `glass_prophets`
- `storm_watcher` → faction: `glass_prophets`

**Tile test:** Rename `data/tile_tests/storm_cults_town.json` → `data/tile_tests/glass_prophets_town.json`, update `faction_territory` inside.

### 1.6 Display Name References in main_questline.json

The questline uses display names with spaces as reputation keys. These must become snake_case IDs:

| Old Key | New Key |
|---------|---------|
| `"Mirror Monks"` | `"mirror_monks"` |
| `"Sand-Engineers"` | `"sand_engineers"` |
| `"Glassborn"` | `"glassborn"` |

Affects: `min_faction_reputation`, `max_faction_reputation`, `faction_reputation_gte` in quest criteria and content injections.

### 1.7 Reputation Keys in NPC Dialogue

NPCs use display names in `faction_reputation_gte` conditions. These must also become snake_case:

| Old Key | New Key | Files |
|---------|---------|-------|
| `"Mirror Monks"` | `"mirror_monks"` | npcs.json (aria_architect) |
| `"Sand-Engineers"` | `"sand_engineers"` | npcs.json (aria_architect) |
| `"Glassborn"` | `"glassborn"` | npcs.json (aria_architect, forge_master_durgan) |
| `"IronCovenant"` | `"iron_covenant"` | npcs.json (mirror_high_prism, forge_master_durgan) |
| `"GlassProphets"` | `"glass_prophets"` | npcs.json |
| `"MirrorMonks"` | `"mirror_monks"` | npcs.json (high_prism) |
| `"SynthesisSeekers"` | `"synthesis_seekers"` | npcs.json (saint_matthias) |
| `"HeliographNetwork"` | `"archive_consciousness"` | npcs.json (aria_architect) |

### 1.8 Rust Source Touchpoints

All hardcoded faction strings in src/. The LeadDeveloper should grep to confirm completeness.

| File | Old String(s) | New String(s) |
|------|--------------|--------------|
| `src/game/faction.rs` | `"MirrorMonks"`, `"SandEngineers"`, `"SaltTraders"`, `"StormCults"`, `"RefractionOutcasts"`, `"ArchiveDrones"` | `"mirror_monks"`, `"sand_engineers"`, `"salt_traders"`, `"glass_prophets"`, `"refraction_outcasts"`, `"archive_consciousness"` |
| `src/game/adaptation.rs` | `"MirrorMonks"`, `"SaltTraders"` | `"mirror_monks"`, `"salt_traders"` |
| `src/game/generation/settlement/mod.rs` | `"MirrorMonks"`, `"SaltTraders"`, `"Glassborn"`, `"StormCults"` | `"mirror_monks"`, `"salt_traders"`, `"glassborn"`, `"glass_prophets"` |
| `src/bin/mapgen_tool.rs` | `"MirrorMonks"`, `"SaltTraders"` | `"mirror_monks"`, `"salt_traders"` |
| `src/game/generation/story.rs` | `"Glassborn Collective"`, `"Naia Glassborn"` | `"glassborn"` (for faction ref), keep `"Naia Glassborn"` (character name) |
| `src/game/book.rs` | `"Glassborn"` | `"glassborn"` (if faction ref; keep if display text) |
| `src/game/systems/quest.rs` | `"Glassborn"` | `"glassborn"` |

Also add the 4 new faction IDs to `faction.rs` reputation initialization: `"synthesis_seekers"`, `"iron_covenant"`, `"glass_prophets"`, `"wandering_court"`.

### 1.9 Save Migration

Existing saves have reputation HashMaps keyed by PascalCase faction IDs. A save migration must:
1. Rename all keys per the §1.2 table
2. If a save has `"StormCults"` reputation, transfer it to `"glass_prophets"`
3. Initialize new faction reputations (`synthesis_seekers`, `iron_covenant`, `glass_prophets` if not from StormCults migration, `wandering_court`) at 0

### 1.10 Files Affected (Complete List)

**Data files:**
- `data/factions.json` — rename all IDs, add 4 new factions
- `data/npcs.json` — rename all `faction` fields, all reputation keys in dialogue conditions
- `data/main_questline.json` — rename all reputation keys in criteria and injections
- `data/quests.json` — rename faction references in reputation rewards
- `data/dialogues.json` — rename all `faction` fields and reputation keys
- `data/traders.json` — rename all `faction` fields
- `data/narrative_integration.json` — rename faction IDs, remove `hermits` from faction_influences
- `data/narrative_templates.json` — rename faction display names in template strings
- `data/structures/structures.json` — rename `metadata.faction`, `metadata.tags`, description text
- `data/recipes.json` — rename `faction_required`
- `data/adaptations.json` — check for faction references (faction_visibility)
- `data/biome_spawn_tables.json` — no faction IDs (NPC IDs only), no changes needed
- `data/tile_tests/*.json` — rename `faction_territory` values, rename storm_cults file

**Rust source:**
- `src/game/faction.rs`
- `src/game/adaptation.rs`
- `src/game/generation/settlement/mod.rs`
- `src/game/generation/story.rs`
- `src/game/book.rs`
- `src/game/systems/quest.rs`
- `src/bin/mapgen_tool.rs`

**Schemas (update to match):**
- `schemas/factions_v1.json`
- `schemas/dialogues_v1.json`
- `schemas/npcs_v1.json`

---

## 2. NPC Deduplication

### 2.1 Canonical IDs

| Duplicate Pair | Canonical ID | Merge From | Rationale |
|---------------|-------------|-----------|-----------|
| `the_architect` / `aria_architect` | `the_architect` | `aria_architect` | main_questline.json references `the_architect`. Merge aria_architect's richer dialogue (8 entries, 3 actions) into the_architect. Set faction to `archive_consciousness`. |
| `custodian_iri_7` / `custodian_iri7` | `custodian_iri_7` | `custodian_iri7` | main_questline.json references `custodian_iri_7`. Merge custodian_iri7's extra dialogue (adaptation branch) and actions (query_archive, data_trade). Keep glyph `I`. Set faction to `archive_consciousness`. |
| `high_prism` / `mirror_high_prism` | **Both survive — different characters** | — | See §2.2 |
| `sable_of_the_seam` / `sable_pathspeaker` | `sable_of_the_seam` | `sable_pathspeaker` | main_questline.json references `sable_of_the_seam`. Merge sable_pathspeaker's richer dialogue (5 entries, crucible_rite, glass_routes). Keep glyph `S`. |

### 2.2 The High Prism / The Prism — Two Characters

These are **not duplicates**. They are two different characters:

| NPC | Identity | Faction | Role |
|-----|---------|---------|------|
| `high_prism` | The High Prism | `mirror_monks` | Orthodox Monk leader. Successor to Brother Halix's political authority. |
| `mirror_high_prism` → **rename to `the_prism`** | The Prism | `glass_prophets` | Glass Prophets leader. Act IV boss (Shard of Soul). Matches lore graph ID `the_prism`. |

**Fix in main_questline.json:** The `shard_of_soul` quest references `high_prism` as the talk_to target. This should be `the_prism` — the Shard of Soul boss fight is against the Glass Prophets leader, not the Orthodox Monk leader.

### 2.3 Remove Test NPC

Delete `merchant_test` from `npcs.json`. It's a test stub with no lore value.

### 2.4 NPC-to-Graph ID Alignment

After dedup, these game NPC IDs still don't match lore graph actor IDs:

| Game NPC ID | Lore Graph Actor ID | Action |
|------------|-------------------|--------|
| `forewoman_ressa` | `ressa_vane` | **No change.** Game uses role-prefixed IDs for NPCs, graph uses personal names. Create a mapping file (see §4). |
| `forge_master_durgan` | `kaine_durgan` | Same — mapping file. |
| `brother_halix` | `brother_halix` | ✓ Already aligned |
| `saint_matthias` | `saint_matthias` | ✓ Already aligned |
| `the_architect` | `the_architect` | ✓ Already aligned (after dedup) |
| `custodian_iri_7` | — | No graph equivalent yet. Add to graph in follow-up. |
| `the_prism` | `the_prism` | ✓ Aligned (after rename) |
| `sable_of_the_seam` | `sable_of_the_seam` | ✓ Aligned (after dedup) |
| `echo_of_memory` | `echo_of_memory` | ✓ Already aligned |
| `dying_pilgrim` | — | No graph equivalent yet. Add to graph in follow-up. |

---

## 3. Lore Graph ID Mapping File

Rather than renaming either side, create a mapping file that the future export pipeline can consume.

**File:** `data/lore_id_map.json`

```json
{
  "description": "Maps game NPC IDs to lore graph actor IDs where they differ",
  "npc_to_graph": {
    "forewoman_ressa": "ressa_vane",
    "forge_master_durgan": "kaine_durgan",
    "captain_vasquez_ghost": "captain_elena_vasquez",
    "dr_kira_thorne": "dr_kira_thorne",
    "unit_seven": "aria",
    "archive_custodian": "custodian_iri_7"
  },
  "faction_game_to_graph": {
    "salt_traders": null,
    "refraction_outcasts": null
  },
  "notes": {
    "salt_traders": "Game-only minor faction. No lore graph equivalent yet.",
    "refraction_outcasts": "Game-only minor faction. No lore graph equivalent yet.",
    "unit_seven": "Game NPC for ARIA's drone avatar. Maps to aria in the graph."
  }
}
```

This file is consumed by the chronicle→game export pipeline (when built). It's not loaded by the game at runtime — it's a build-time reference.

---

## 4. Locations Schema

The game needs a `data/locations.json` to define named settlements and quest locations. Proposed schema:

```json
{
  "$schema": "locations_v1",
  "locations": [
    {
      "id": "last_salt",
      "name": "Last Salt",
      "type": "settlement",
      "biome": "saltflat",
      "description": "Starting settlement built around a functioning brine well.",
      "faction_presence": ["mirror_monks", "sand_engineers", "salt_traders"],
      "services": ["trade", "rest", "quest"],
      "npcs": ["dying_pilgrim", "merchant_keth", "mirror_monk"],
      "lore_graph_id": "last_salt"
    }
  ]
}
```

**Fields:**
- `id` — snake_case, unique. Matches lore graph place ID where applicable.
- `name` — display name.
- `type` — one of: `settlement`, `dungeon`, `landmark`, `ruin`.
- `biome` — which biome this location spawns in (references biome_profiles.json).
- `description` — short in-game description.
- `faction_presence` — which factions have a presence here (affects NPC spawns, structure selection).
- `services` — what the player can do here.
- `npcs` — named NPCs that spawn here (overrides biome spawn tables).
- `lore_graph_id` — optional. Maps to the chronicle graph place ID for the export pipeline.

**Initial data** (from lore graph + quest spine):

| ID | Name | Type | Biome | Faction Presence |
|----|------|------|-------|-----------------|
| `last_salt` | Last Salt | settlement | saltflat | mirror_monks, sand_engineers, salt_traders |
| `vitrified_library` | The Vitrified Library Wedge | dungeon | ruins | mirror_monks |
| `crucible_block` | The Crucible Block | dungeon | shattered_citadel | sand_engineers |
| `deep_archive_wing` | The Deep Archive Wing | dungeon | shattered_citadel | archive_consciousness |
| `vector_spire` | The Vector Spire | landmark | refraction_fields | — |
| `angle_cathedra` | Angle Cathedra | settlement | glass_gardens | mirror_monks |
| `nexus_plateau` | The Nexus Plateau | landmark | saltflat | synthesis_seekers, salt_traders |

The Creative Director will provide the full initial data file in the content follow-up.

---

## 5. Execution Order

1. **LeadDeveloper: Faction rename** (§1) — all data files + Rust source + save migration. Single commit.
2. **LeadDeveloper: NPC dedup** (§2) — merge duplicates, remove merchant_test, fix main_questline references. Single commit.
3. **LeadDeveloper: Add lore_id_map.json** (§3) — new file, no code changes. Single commit.
4. **LeadDeveloper: Add locations.json** (§4) — new file + schema. Single commit. (Creative Director provides initial data.)
5. **Creative Director: Content follow-up** — quest rewrites, new recipes, item renames, skill renames, missing quest items, description improvements. Separate changeset.
6. **Creative Director: Lore graph alignment** — add missing NPCs (dying_pilgrim, custodian_iri_7), add salt_traders and refraction_outcasts as minor factions if desired, update any IDs that changed.

---

## 6. Verification Checklist

After steps 1–4, verify:
- [ ] `cargo build` succeeds with zero warnings
- [ ] `cargo test` — all DES scenarios pass
- [ ] `cargo test --test des_scenarios` — 134 passing (or more)
- [ ] Every faction ID in every JSON file is snake_case
- [ ] `factions.json` has exactly 10 factions (7 renamed + 3 new; StormCults removed, glass_prophets added)
- [ ] No NPC in `npcs.json` has a `faction` value that doesn't exist in `factions.json` (except `independent` and `Unaffiliated`)
- [ ] `main_questline.json` references only canonical NPC IDs: `the_architect`, `custodian_iri_7`, `sable_of_the_seam`, `the_prism`, `brother_halix`, `forewoman_ressa`, `dying_pilgrim`, `high_prism`
- [ ] No duplicate NPC IDs in `npcs.json`
- [ ] `merchant_test` removed
- [ ] Save migration handles PascalCase → snake_case for all reputation keys
