---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# Proposal: Structured Lore as Entity-Relation Graphs

> Status: PROPOSAL
> Date: 2026-04-04
> Author: Elias + Kiro CLI (system-agent)

---

## 1. The Problem

The narrative content in `docs/narrative/` is ~40 markdown files totaling ~300KB of prose. It was generated in batches by AI agents, and it shows two problems:

**Inconsistency.** Different documents contradict each other because they were written in separate sessions without shared context. A faction's motivation in one story doesn't match its description in another. Timeline events shift. Character relationships are stated differently depending on which document you read. The world stops making sense.

**Disconnection from gameplay.** The lore exists as prose documents that agents read for "tone" but can't query structurally. When an agent needs to know "which factions are hostile to each other?" or "what happened before the Schism Wars?", it has to read and interpret prose. It can hallucinate the answer. There's no machine-readable source of truth.

The same problem exists for technical documentation (stale docs misleading agents), but the lore problem is more visible to the player — inconsistent world-building breaks immersion in a way that a stale API doc doesn't.

---

## 2. The Proposal

Replace disconnected prose documents with **structured entity-relation graphs** as the canonical lore source. Every piece of world-building — characters, factions, events, locations, items, concepts — is a node in a graph with typed, directional relationships to other nodes.

### 2.1 What is a node?

```
Entity: "Mirror Monks"
  type: faction
  properties:
    philosophy: "Light reveals truth; refraction is sacred"
    territory: "Refraction Cathedra"
    founded_after: "The Schism Wars"
  relations:
    - hostile_to: "Storm Cults"
    - allied_with: "Archive Drones" (conditional: "when Archive interests align")
    - controls: "Refraction Cathedra"
    - worships: "The Prime Lens"
    - founded_by: "First Lens-Bearer Solara"
```

### 2.2 What is a relation?

A typed, directional edge between two entities. Relations have:
- **Type**: `hostile_to`, `allied_with`, `founded_by`, `located_in`, `caused`, `preceded`, `created`, `member_of`, etc.
- **Direction**: A `hostile_to` B doesn't imply B `hostile_to` A (asymmetric hostility exists).
- **Optional properties**: conditions, time periods, strength, source document.

### 2.3 Entity types

| Type | Examples | Typical relations |
|------|----------|-------------------|
| `character` | First Lens-Bearer Solara, Dr. Kira Thorne | member_of, created, discovered, killed_by |
| `faction` | Mirror Monks, Storm Cults, Salt Traders | hostile_to, allied_with, controls, founded_by |
| `location` | Refraction Cathedra, Singing Dunes, Brine Under | located_in, controlled_by, contains |
| `event` | The Schism Wars, Day of Broken Mirrors, White Noon | caused, preceded, followed_by, involved |
| `concept` | Refraction, Void Energy, Crystal Resonance | related_to, manifests_as, discovered_by |
| `item` | Storm Glass, Prime Lens, ARIA terminals | created_by, found_at, used_by |
| `creature` | Refraction Wraith, Glass Scarab | inhabits, spawned_by, hostile_to |
| `era` | Heliograph Era, Post-Schism, Present Day | preceded, followed_by, defined_by |

### 2.4 What this enables

**Consistency enforcement.** If faction A is `hostile_to` faction B, every quest, dialogue, and encounter involving both factions can query this relationship. An agent generating dialogue for a Mirror Monk NPC can query: "who are the Mirror Monks hostile to?" and get a definitive answer, not a hallucinated one.

**Temporal coherence.** Events have `preceded`/`followed_by` relations. The timeline is a graph, not prose. An agent can query: "what events happened during the Schism Wars?" and get a consistent list.

**Gameplay integration.** The graph can be loaded at runtime. Faction relationships drive NPC behavior. Location properties drive generation. Item lore is queryable in-game (books, examine text). The lore isn't just flavor — it's data.

**Contradiction detection.** If two nodes claim contradictory relationships (faction A is both `allied_with` and `hostile_to` faction B without a temporal qualifier), the graph validator catches it.

---

## 3. Implementation: A Rust Crate

### 3.1 Why a crate?

- **Queryable at build time and runtime.** The graph is loaded by the game for NPC dialogue, faction behavior, item descriptions. It's also queryable by agents during development.
- **Validation.** The crate enforces schema rules: required relations per entity type, no dangling references, temporal consistency.
- **Separation of concerns.** Lore data is not game code. A separate crate keeps the boundary clean.
- **Reusability.** The graph structure is generic — entity types and relation types are configurable. Other projects (or other games) could use the same crate with different schemas.

### 3.2 Core API sketch

```rust
// Define the graph
let mut graph = LoreGraph::new();

// Add entities
graph.add_entity("mirror_monks", EntityType::Faction, props! {
    "philosophy" => "Light reveals truth",
    "territory" => "refraction_cathedra",
});

// Add relations
graph.add_relation("mirror_monks", "storm_cults", RelationType::HostileTo, None);
graph.add_relation("schism_wars", "mirror_monks", RelationType::Founded, None);

// Query
let enemies = graph.related_to("mirror_monks", RelationType::HostileTo);
let timeline = graph.events_in_order("heliograph_era", "present_day");
let faction_web = graph.subgraph_around("mirror_monks", depth: 2);

// Validate
let issues = graph.validate(); // dangling refs, contradictions, missing required relations
```

### 3.3 Data format

The graph is defined in a human-readable format (RON, TOML, or a custom DSL) that lives in `data/lore/` alongside the game's other data files. Not markdown — structured data.

```ron
// data/lore/factions.ron
(
    entities: [
        (
            id: "mirror_monks",
            entity_type: Faction,
            name: "Mirror Monks",
            properties: {
                "philosophy": "Light reveals truth; refraction is sacred",
                "territory": "refraction_cathedra",
            },
        ),
    ],
    relations: [
        (from: "mirror_monks", to: "storm_cults", relation: HostileTo),
        (from: "mirror_monks", to: "archive_drones", relation: AlliedWith,
         properties: { "condition": "when Archive interests align" }),
    ],
)
```

### 3.4 Prose generation from graph

The existing prose documents don't disappear — they become *generated views* of the graph. A template system produces readable lore from structured data:

```
Template: "{{name}} is a {{type}} that {{philosophy}}. They control {{territory}} 
           and are hostile to {{enemies}}."

Output: "The Mirror Monks are a faction that believes light reveals truth. 
         They control the Refraction Cathedra and are hostile to the Storm Cults."
```

In-game books, examine text, and NPC dialogue pull from the graph, not from static prose. This means lore is always consistent with the graph — because it's derived from it.

---

## 4. Timing Feedback (system-agent)

### When to do this

**Not now.** Here's why:

1. The dead code cleanup and architecture refactor are higher priority. They affect every development session. Lore inconsistency is a content quality issue — important, but not blocking development.

2. The crate needs design time. Entity types, relation types, validation rules, query API — these need to be thought through carefully. Rushing it produces another scaffold-and-abandon artifact.

3. The existing lore is ~300KB of prose. Converting it to a graph is a significant content migration. Every entity needs to be extracted, every relationship identified, every contradiction resolved. This is weeks of work, not days.

### Suggested timeline

| When | What |
|------|------|
| **Now** | Write this proposal. Socialize the idea. |
| **After Phase 0.5** (dead code cleanup) | Review the existing lore for obvious contradictions. Create a simple `data/lore/factions.ron` with just faction relationships as a proof of concept — no crate, just a data file the game loads. |
| **After architecture refactor Phase 1** | If the proof of concept works, design the crate API. Start with the smallest useful scope: factions + locations + their relationships. |
| **After Phase 3** (combat/movement refactored) | Build the crate. Migrate faction and location data. Wire into NPC dialogue and faction behavior. |
| **Ongoing** | Migrate remaining lore (characters, events, items, history) incrementally. Each migration pass converts one entity type from prose to graph. |

### The proof of concept is key

Before building a crate, test the idea with a single RON file:

```
data/lore/factions.ron → loaded by DataLoader → used by NPC dialogue system
```

If faction relationships from the graph produce better, more consistent NPC behavior than the current hardcoded faction data in `data/factions.json`, the concept is proven. If it doesn't add value, you've spent one file's worth of effort, not a crate.

---

## 5. On Technical Documentation Staleness

The commit hash + archive approach you mentioned is the right lightweight fix for technical docs:

```markdown
---
created: 2025-12-24
last_verified: 2026-04-04
commit: a1b2c3d
status: current | stale | archived
---
```

A simple hook or script can flag docs where `last_verified` is older than N days or where `commit` is more than M commits behind HEAD. This doesn't require a graph — it's metadata on existing files.

The graph approach for technical docs (e.g., "module A depends on module B, which was documented in doc C") is interesting but significantly more complex. The system status registry (`SYSTEM_STATUS.md`) already serves as a lightweight version of this for gameplay systems. I'd defer the technical doc graph until the lore graph proves the concept.
