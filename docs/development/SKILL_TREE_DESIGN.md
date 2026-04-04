---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# Skill Tree Design

> Status: Draft | Last updated: 2026-03-07

## Overview

The skill system in Saltglass Steppe consists of 7 categories with approximately 90 skills total (~13 per category). Skills are data-driven through JSON configuration and rendered in a tree graph UI with canvas-based rendering.

**Categories**: SaltAlchemy, Crafting, Social, Survival, Medical, MeleeCombat, RangedCombat

**Design Philosophy**: Skills reflect the harsh crystalline desert environment where glass storms reshape the world, mutations carry social consequences, and survival requires mastering both ancient techniques and scavenged technology.

## Design Rules

### Cost Guidelines
- **Root skills**: 1-2 points
- **Tier 2**: 2-3 points  
- **Tier 3**: 3-4 points
- **Tier 4**: 4-5 points
- **Master skills**: 5-6 points

### Level Guidelines
- Most skills: `max_level: 1` (binary unlock)
- Scaling skills: `max_level: 3-5` (damage, accuracy, efficiency)
- Never exceed `max_level: 5`

### Active vs Passive
- **Active**: Require player input, consume resources/time
- **Passive**: Always-on bonuses, calculated in `recalculate_passive_bonuses()`
- Target: 1-2 active skills per category

### Blocked Policy
Skills marked `blocked: true` appear in UI as locked but aren't integrated into game systems. Used for skills requiring unimplemented systems.

---

## Category: SaltAlchemy

*"The old ways of transmutation, adapted for a vitrified world."*

### Tree Structure
```
crucible_techniques (3) ──┬── adaptation_tinctures (1)
                          ├── poison_creation (1) ── contact_poison (1) [BLOCKED]
                          └── glass_fusion (1) ── storm_glass_working (1) [BLOCKED]

brine_distillation (3) ───┬── salt_preservation (1)
                          ├── void_reagents (1) ── void_channeling (1) [BLOCKED]
                          └── crystal_growing (1)

salt_communion (1) ───────┬── pilgrim_rites (1)
                          └── caste_marking (1) [BLOCKED]

refraction_brewing (1) ───┬── light_tinctures (1)
                          └── prism_elixirs (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| crucible_techniques | Crucible Techniques | 3 | 2 | passive | Existing root skill |
| adaptation_tinctures | Adaptation Tinctures | 1 | 2 | passive | Existing child skill |
| poison_creation | Poison Creation | 1 | 2 | passive | Existing child skill |
| contact_poison | Contact Poison | 1 | 3 | active | Existing, BLOCKED |
| glass_fusion | Glass Fusion | 1 | 3 | passive | Fuse glass shards into tools |
| storm_glass_working | Storm Glass Working | 1 | 4 | passive | BLOCKED - requires storm system |
| brine_distillation | Brine Distillation | 3 | 2 | passive | Existing root skill |
| salt_preservation | Salt Preservation | 1 | 2 | passive | Existing child skill |
| void_reagents | Void Reagents | 1 | 3 | passive | Existing child skill |
| void_channeling | Void Channeling | 1 | 4 | active | BLOCKED - requires void energy system |
| crystal_growing | Crystal Growing | 1 | 3 | passive | Grow salt crystals for tools |
| salt_communion | Salt Communion | 1 | 2 | passive | Understand salt-touched creatures |
| pilgrim_rites | Pilgrim Rites | 1 | 3 | active | Ritual bonuses at shrines |
| caste_marking | Caste Marking | 1 | 4 | passive | BLOCKED - requires faction bleed |
| refraction_brewing | Refraction Brewing | 1 | 2 | passive | Brew light-based consumables |
| light_tinctures | Light Tinctures | 1 | 3 | passive | Enhance vision/accuracy temporarily |
| prism_elixirs | Prism Elixirs | 1 | 4 | active | BLOCKED - requires light combat system |

### Blockers
- **contact_poison**: Requires poison application system
- **storm_glass_working**: Requires storm system implementation
- **void_channeling**: Requires void energy mechanics
- **caste_marking**: Requires faction reputation bleed system
- **prism_elixirs**: Requires light-based combat mechanics

### Balancing Notes
- High-tier alchemy skills should require rare reagents found in dangerous areas
- Void-based skills represent late-game power with social consequences
- Synergizes with Survival (reagent gathering) and Medical (healing compounds)

---

## Category: Crafting

*"Scavenged technology and ancient techniques, fused by necessity."*

### Tree Structure
```
scrap_salvage (1) ────────── expert_salvage (1)

weapon_modding (1) ───────┬── precision_tuning (1)
                          └── jury_rigging (1)

ammo_fabrication (1) ─────┬── specialized_rounds (1)
                          └── explosive_charges (1)

glass_working (1) ────────┬── lens_grinding (1)
                          └── mirror_polishing (1)

relic_restoration (1) ────┬── archive_techniques (1)
                          └── saint_relics (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| scrap_salvage | Scrap Salvage | 1 | 1 | passive | Existing root skill |
| expert_salvage | Expert Salvage | 1 | 2 | passive | Existing child skill |
| weapon_modding | Weapon Modding | 1 | 2 | passive | Existing root skill |
| precision_tuning | Precision Tuning | 1 | 3 | passive | Improve weapon accuracy |
| jury_rigging | Jury Rigging | 1 | 3 | active | Emergency repairs in field |
| ammo_fabrication | Ammo Fabrication | 1 | 2 | passive | Existing root skill |
| specialized_rounds | Specialized Rounds | 1 | 3 | passive | Craft glass-piercing, salt rounds |
| explosive_charges | Explosive Charges | 1 | 4 | active | Craft demolition tools |
| glass_working | Glass Working | 1 | 2 | passive | Shape glass into tools/weapons |
| lens_grinding | Lens Grinding | 1 | 3 | passive | Craft optical equipment |
| mirror_polishing | Mirror Polishing | 1 | 3 | passive | Create reflective surfaces |
| relic_restoration | Relic Restoration | 1 | 2 | passive | Repair ancient artifacts |
| archive_techniques | Archive Techniques | 1 | 3 | passive | Understand pre-storm technology |
| saint_relics | Saint Relics | 1 | 5 | passive | BLOCKED - requires relic system |

### Blockers
- **saint_relics**: Requires saint relic mechanics and faction systems

### Balancing Notes
- Glass-working skills should synergize with light-based combat
- Relic restoration provides access to unique pre-storm technology
- Explosive skills balanced by resource scarcity and danger

---

## Category: Social

*"In the wasteland, words can be sharper than glass."*

### Tree Structure
```
faction_lore (1) ─────────── faction_expertise (1)

bartering (1) ────────────── hard_bargaining (1)

intimidation (1) ─────────┬── reputation_leverage (1)
                          └── caste_authority (1) [BLOCKED]

diplomacy (1) ────────────┬── neutral_ground (1)
                          └── peace_broker (1)

pilgrim_tongue (1) ───────┬── ancient_dialects (1)
                          └── scripture_reading (1)

outcaste_bonds (1) ───────── void_touched_kinship (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| faction_lore | Faction Lore | 1 | 1 | passive | Existing root skill |
| faction_expertise | Faction Expertise | 1 | 2 | passive | Existing child skill |
| bartering | Bartering | 1 | 1 | passive | Existing root skill |
| hard_bargaining | Hard Bargaining | 1 | 2 | passive | Existing child skill |
| intimidation | Intimidation | 1 | 2 | passive | Existing root skill |
| reputation_leverage | Reputation Leverage | 1 | 3 | passive | Use faction standing in negotiations |
| caste_authority | Caste Authority | 1 | 4 | passive | BLOCKED - requires caste system |
| diplomacy | Diplomacy | 1 | 2 | passive | Peaceful conflict resolution |
| neutral_ground | Neutral Ground | 1 | 3 | active | Establish temporary truces |
| peace_broker | Peace Broker | 1 | 4 | passive | Mediate faction disputes |
| pilgrim_tongue | Pilgrim Tongue | 1 | 2 | passive | Understand religious terminology |
| ancient_dialects | Ancient Dialects | 1 | 3 | passive | Read pre-storm inscriptions |
| scripture_reading | Scripture Reading | 1 | 3 | passive | Interpret religious texts |
| outcaste_bonds | Outcaste Bonds | 1 | 2 | passive | Connect with other outcasts |
| void_touched_kinship | Void Touched Kinship | 1 | 4 | passive | BLOCKED - requires mutation system |

### Blockers
- **caste_authority**: Requires caste/social hierarchy system
- **void_touched_kinship**: Requires mutation/adaptation social consequences

### Balancing Notes
- Social skills should provide alternative solutions to combat encounters
- Religious/pilgrim skills unlock unique dialogue options and quest paths
- Outcaste skills become more valuable as player gains mutations

---

## Category: Survival

*"The steppe takes everything. Learn to take back."*

### Tree Structure
```
scavenging (1) ───────────── expert_scavenging (1)

track_and_trace (1) ──────┬── predator_sense (1)
                          └── storm_tracking (1) [BLOCKED]

dust_walking (1) ─────────┬── glass_stride (1)
                          └── void_step (1) [BLOCKED]

biome_lore (1) ───────────┬── hazard_recognition (1)
                          └── resource_mapping (1)

endurance_training (1) ───┬── heat_adaptation (1)
                          └── thirst_tolerance (1) [BLOCKED]

shelter_craft (1) ────────── storm_shelter (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| scavenging | Scavenging | 1 | 1 | passive | Existing root skill |
| expert_scavenging | Expert Scavenging | 1 | 2 | passive | Existing child skill |
| track_and_trace | Track and Trace | 1 | 2 | passive | Existing root skill |
| predator_sense | Predator Sense | 1 | 3 | passive | Detect hostile creatures |
| storm_tracking | Storm Tracking | 1 | 4 | passive | BLOCKED - requires storm system |
| dust_walking | Dust Walking | 1 | 2 | passive | Existing root skill |
| glass_stride | Glass Stride | 1 | 3 | passive | Move safely over glass terrain |
| void_step | Void Step | 1 | 4 | active | BLOCKED - requires void energy |
| biome_lore | Biome Lore | 1 | 1 | passive | Existing root skill |
| hazard_recognition | Hazard Recognition | 1 | 2 | passive | Identify environmental dangers |
| resource_mapping | Resource Mapping | 1 | 3 | passive | Locate valuable materials |
| endurance_training | Endurance Training | 1 | 2 | passive | Resist fatigue and exposure |
| heat_adaptation | Heat Adaptation | 1 | 3 | passive | Reduced heat damage |
| thirst_tolerance | Thirst Tolerance | 1 | 3 | passive | BLOCKED - requires thirst system |
| shelter_craft | Shelter Craft | 1 | 2 | passive | Build temporary shelters |
| storm_shelter | Storm Shelter | 1 | 4 | active | BLOCKED - requires storm system |

### Blockers
- **storm_tracking**: Requires storm prediction/tracking mechanics
- **void_step**: Requires void energy and teleportation system
- **thirst_tolerance**: Requires thirst/dehydration mechanics
- **storm_shelter**: Requires storm system and shelter mechanics

### Balancing Notes
- Movement skills should provide tactical advantages in exploration
- Environmental adaptation skills become crucial in harsh biomes
- Resource skills synergize with Crafting and SaltAlchemy

---

## Category: Medical

*"Flesh and glass, salt and bone—all can be mended."*

### Tree Structure
```
basic_medical_practice (1) ─┬── wound_packing (1) [active]
                            ├── triage (1)
                            └── field_surgery (1)

stimulant_use (1) ──────────┬── combat_stims (1)
                            └── endurance_boosters (1)

mutation_medicine (1) ──────┬── adaptation_therapy (1) [BLOCKED]
                            └── void_toxin_treatment (1) [BLOCKED]

glass_wound_care (1) ───────┬── shard_extraction (1)
                            └── refraction_scarring (1) [BLOCKED]

pilgrim_healing (1) ────────── saint_touch (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| basic_medical_practice | Basic Medical Practice | 1 | 1 | passive | Existing root skill |
| wound_packing | Wound Packing | 1 | 2 | active | Existing child skill |
| triage | Triage | 1 | 2 | passive | Existing child skill |
| field_surgery | Field Surgery | 1 | 3 | active | Emergency surgical procedures |
| stimulant_use | Stimulant Use | 1 | 2 | passive | Existing root skill |
| combat_stims | Combat Stims | 1 | 3 | active | Temporary combat bonuses |
| endurance_boosters | Endurance Boosters | 1 | 3 | active | Resist fatigue and exposure |
| mutation_medicine | Mutation Medicine | 1 | 2 | passive | Understand adaptation effects |
| adaptation_therapy | Adaptation Therapy | 1 | 4 | active | BLOCKED - requires mutation system |
| void_toxin_treatment | Void Toxin Treatment | 1 | 4 | active | BLOCKED - requires void poisoning |
| glass_wound_care | Glass Wound Care | 1 | 2 | passive | Treat glass-related injuries |
| shard_extraction | Shard Extraction | 1 | 3 | active | Remove embedded glass safely |
| refraction_scarring | Refraction Scarring | 1 | 4 | passive | BLOCKED - requires mutation system |
| pilgrim_healing | Pilgrim Healing | 1 | 3 | passive | Religious healing techniques |
| saint_touch | Saint Touch | 1 | 5 | active | BLOCKED - requires saint relic system |

### Blockers
- **adaptation_therapy**: Requires mutation/adaptation system
- **void_toxin_treatment**: Requires void poisoning mechanics
- **refraction_scarring**: Requires mutation system and scarring mechanics
- **saint_touch**: Requires saint relic system and miraculous healing

### Balancing Notes
- Medical skills should provide alternatives to consumable healing items
- Glass-specific medical skills reflect the unique hazards of the setting
- Religious healing skills require pilgrimage or faction standing

---

## Category: MeleeCombat

*"In close quarters, glass cuts both ways."*

### Tree Structure
```
glass_fighting (1) ───────── seam_breaker (1)

angle_reading (1) ────────┬── perfect_strike (1)
                          └── defensive_geometry (1)

vortex_footwork (1) ──────┬── whirlwind_stance (1)
                          └── storm_dance (1) [BLOCKED]

salt_flurry (1) [active] ─┬── crystalline_combo (1) [active]
                          └── brine_strike (1) [active]

void_blade_arts (1) ──────┬── shadow_cut (1) [BLOCKED]
                          └── reality_rend (1) [BLOCKED]

pilgrim_combat (1) ───────── saint_weapon_mastery (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| glass_fighting | Glass Fighting | 1 | 2 | passive | Existing root skill |
| seam_breaker | Seam Breaker | 1 | 3 | passive | Existing child skill |
| angle_reading | Angle Reading | 1 | 2 | passive | Existing root skill |
| perfect_strike | Perfect Strike | 1 | 3 | passive | Increased critical hit chance |
| defensive_geometry | Defensive Geometry | 1 | 3 | passive | Improved dodge against melee |
| vortex_footwork | Vortex Footwork | 1 | 2 | passive | Existing root skill |
| whirlwind_stance | Whirlwind Stance | 1 | 3 | active | Attack multiple adjacent enemies |
| storm_dance | Storm Dance | 1 | 4 | active | BLOCKED - requires storm system |
| salt_flurry | Salt Flurry | 1 | 2 | active | Existing root skill |
| crystalline_combo | Crystalline Combo | 1 | 3 | active | Multi-hit attack sequence |
| brine_strike | Brine Strike | 1 | 3 | active | Corrosive melee attack |
| void_blade_arts | Void Blade Arts | 1 | 3 | passive | Harness void energy in combat |
| shadow_cut | Shadow Cut | 1 | 4 | active | BLOCKED - requires void energy |
| reality_rend | Reality Rend | 1 | 5 | active | BLOCKED - requires void energy |
| pilgrim_combat | Pilgrim Combat | 1 | 2 | passive | Religious martial techniques |
| saint_weapon_mastery | Saint Weapon Mastery | 1 | 4 | passive | BLOCKED - requires saint relics |

### Blockers
- **storm_dance**: Requires storm system and environmental interaction
- **shadow_cut**: Requires void energy mechanics
- **reality_rend**: Requires advanced void energy system
- **saint_weapon_mastery**: Requires saint relic weapon system

### Balancing Notes
- Glass-based skills should have high damage but risk self-injury
- Void skills represent high power with potential corruption consequences
- Active skills should consume stamina or have cooldowns

---

## Category: RangedCombat

*"Distance is survival. Precision is art."*

### Tree Structure
```
draw_a_bead (1) ──────────── snipers_eye (1)

ammo_conservation (1) ────┬── efficient_shooting (1)
                          └── scrap_rounds (1)

brine_volley (1) ─────────┬── salt_spray (1) [active]
                          └── corrosive_coating (1)

void_aim (1) ─────────────┬── phase_shot (1) [BLOCKED]
                          └── reality_pierce (1) [BLOCKED]

light_weaponry (1) ───────┬── beam_focus (1) [BLOCKED]
                          └── prism_scatter (1) [BLOCKED]

pilgrim_archery (1) ──────── saint_blessed_shots (1) [BLOCKED]
```

### Skills

| ID | Name | Lv | Cost | Type | Notes |
|----|------|----|----|------|-------|
| draw_a_bead | Draw a Bead | 1 | 2 | passive | Existing root skill |
| snipers_eye | Sniper's Eye | 1 | 3 | passive | Existing child skill |
| ammo_conservation | Ammo Conservation | 1 | 2 | passive | Existing root skill |
| efficient_shooting | Efficient Shooting | 1 | 3 | passive | Reduced ammo consumption |
| scrap_rounds | Scrap Rounds | 1 | 3 | passive | Craft improvised ammunition |
| brine_volley | Brine Volley | 1 | 2 | passive | Existing root skill |
| salt_spray | Salt Spray | 1 | 3 | active | Area-effect ranged attack |
| corrosive_coating | Corrosive Coating | 1 | 3 | passive | Ammunition causes ongoing damage |
| void_aim | Void Aim | 1 | 2 | passive | Existing root skill |
| phase_shot | Phase Shot | 1 | 4 | active | BLOCKED - requires void energy |
| reality_pierce | Reality Pierce | 1 | 5 | active | BLOCKED - requires void energy |
| light_weaponry | Light Weaponry | 1 | 3 | passive | Use light-based ranged weapons |
| beam_focus | Beam Focus | 1 | 4 | active | BLOCKED - requires light combat |
| prism_scatter | Prism Scatter | 1 | 4 | active | BLOCKED - requires light combat |
| pilgrim_archery | Pilgrim Archery | 1 | 2 | passive | Religious ranged techniques |
| saint_blessed_shots | Saint Blessed Shots | 1 | 4 | passive | BLOCKED - requires saint system |

### Blockers
- **phase_shot**: Requires void energy and phase mechanics
- **reality_pierce**: Requires advanced void energy system
- **beam_focus**: Requires light-based combat system
- **prism_scatter**: Requires light-based combat system
- **saint_blessed_shots**: Requires saint blessing system

### Balancing Notes
- Ranged skills should emphasize positioning and resource management
- Light-based weapons represent advanced technology requiring rare components
- Void skills offer power at the cost of potential corruption

---

## Cross-Category Synergies

### Major Synergy Clusters

**Alchemist-Medic**: SaltAlchemy + Medical
- Craft healing compounds and antidotes
- Understand mutation effects and treatments
- Create specialized medical supplies

**Scavenger-Crafter**: Survival + Crafting  
- Locate rare materials in dangerous areas
- Repair equipment in the field
- Maximize resource efficiency

**Warrior-Diplomat**: Combat + Social
- Intimidation backed by combat reputation
- Negotiate from position of strength
- Understand faction military capabilities

**Pilgrim-Scholar**: Social + SaltAlchemy
- Access to religious alchemical traditions
- Understand ancient techniques and lore
- Gain faction-specific knowledge

### Implementation Notes
- Synergies should emerge naturally from skill combinations
- No explicit "synergy bonuses" - let players discover interactions
- Some high-tier skills may require prerequisites from multiple categories

---

## Implementation Priority

### Phase 1: Core Skills (Immediate)
All existing skills (35 total) are already implemented and functional.

### Phase 2: Basic Extensions (Next Sprint)
- **Crafting**: precision_tuning, jury_rigging, glass_working
- **Social**: diplomacy, neutral_ground, pilgrim_tongue  
- **Survival**: predator_sense, hazard_recognition, endurance_training
- **Medical**: field_surgery, combat_stims, glass_wound_care

### Phase 3: Advanced Skills (Future)
- Skills requiring new systems (storm, void energy, mutations)
- Cross-category prerequisite skills
- Master-tier abilities (5+ point cost)

### Phase 4: Blocked Skills (System-Dependent)
- Storm system skills: storm_tracking, storm_dance, storm_shelter
- Void energy skills: void_channeling, phase_shot, shadow_cut
- Mutation system skills: adaptation_therapy, refraction_scarring
- Faction system skills: caste_authority, saint_relics

### Technical Implementation Notes

**Passive Effects**: Use consistent `effect_type` naming:
- `melee_accuracy_bonus` (not `accuracy_bonus`)
- `glass_damage_resistance` (not `damage_resistance`)
- `scavenging_efficiency_bonus` (specific to system)

**Active Skills**: Dispatch on `effect_type` from JSON:
- Implement in relevant system modules
- Handle resource costs (stamina, materials, cooldowns)
- Provide clear feedback on success/failure

**UI Integration**: 
- Tree layout uses `tree_parent` for visual hierarchy
- Cross-tree prerequisites in `prerequisites` array
- Blocked skills show as locked with tooltip explanation

---

*End of Document*