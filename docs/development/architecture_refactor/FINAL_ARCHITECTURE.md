# Final Architecture: VERA — Verified Effect-Rule Architecture

> VERA: Pure rule functions describe what should change (effects), the system applies them mechanically, and a trace records everything that happened for verification.

> Status: FINAL — supersedes ESCAEV v1, v2, and reconsideration
> Date: 2026-04-04
> Authors: Elias + Kiro CLI (system-agent)
> Inputs: ESCAEV v1/v2, two LeadDeveloper reviews, reconsideration, codebase health audit, DES analysis

---

## 1. Design Principles

### 1.1 Why this pattern exists

One constraint dominates: AI agents must self-verify their work. The scaffold-and-abandon pattern produced ~3,600 LOC of dead code because there was no structural gate between "code compiles" and "feature works in gameplay." The architecture must make unwired code structurally visible.

### 1.2 Design decisions

**VERA** stands for **Verified Effect-Rule Architecture**. The name captures the three pillars: rules produce effects, effects are verified (by traces at test time, by exhaustive match at compile time), and the architecture enforces this flow structurally.

1. **Rust-idiomatic over novel.** Every construct maps to standard Rust patterns that LLMs have extensive training data for: enums, match, free functions, `impl` blocks, `From`/`Into` traits. No custom DSLs, no proc macros, no trait objects for core dispatch.

2. **Elm Architecture as the core.** `(State, Command) → Vec<Effect>` via pure functions. This is the most well-understood functional architecture in LLM training data. We call the pure functions "rules" but they're just functions.

3. **Effects are enums, not trait objects.** Domain-scoped: `Effect::Combat(CombatEffect::DealDamage { ... })`. This gives exhaustive match checking — the compiler catches unhandled variants. AI agents can grep for all handlers of a given effect.

4. **Traces are test infrastructure, not runtime overhead.** Ephemeral, opt-in, only materialized during DES runs. Zero cost in normal gameplay.

5. **Reactions are deferred, not immediate.** Effects go into a queue. After primary effects are applied, reactions run. This avoids re-entrancy and keeps the borrow checker happy — you never need `&mut state` while iterating over effects.

6. **Incremental migration.** Each phase produces a working codebase. Legacy methods coexist with rule-based methods during migration. No flag day.

### 1.3 Rust idiom alignment

| Pattern element | Rust idiom | LLM training signal |
|----------------|-----------|-------------------|
| Command enum | `enum Action { Move { dx, dy }, UseItem { idx }, ... }` | Universal Rust pattern |
| Effect enum | `enum Effect { Combat(CombatEffect), Item(ItemEffect), ... }` | Nested enums, very common |
| Rule function | `fn rule_use_item(cmd, ctx, rng) -> Vec<Effect>` | Free function with explicit args |
| Apply function | `match effect { Effect::Combat(e) => match e { ... } }` | Exhaustive match |
| QueryContext | `struct Ctx<'a> { player: &'a Player, world: &'a World }` | Borrowed struct, lifetime elision |
| Trace | `Vec<TraceEntry>` | Just a vec |
| Reaction | `fn on_enemy_killed(effect, ctx, rng) -> Vec<Effect>` | Same signature as rule |

Every line of code following this pattern looks like normal Rust. An LLM generating code for "a function that takes player state and returns what should change" will naturally produce something close to a rule function. This is the key insight — the pattern should be *invisible* to an agent that already knows Rust.

---

## 2. Core Types

All types live in `src/game/effects/mod.rs` (new module).

### 2.1 Effects

```rust
/// Top-level effect enum. Every state mutation goes through this.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Combat(CombatEffect),
    Item(ItemEffect),
    Player(PlayerEffect),
    Map(MapEffect),
    Quest(QuestEffect),
    Storm(StormEffect),
    Status(StatusEffect),
    Resource(ResourceEffect),
    Event(EventEffect),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatEffect {
    DealDamage { target: Target, amount: i32, source: String },
    ApplyStatus { target: Target, effect: String, duration: i32 },
    Miss { target: Target },
    Kill { target: Target },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemEffect {
    Consume { item_id: String, inventory_index: usize },
    AddToInventory { item_id: String },
    RemoveFromInventory { index: usize },
    SpawnOnMap { item_id: String, x: i32, y: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEffect {
    Heal { amount: i32 },
    SpendAp { amount: i32 },
    GainXp { amount: u32 },
    LevelUp { new_level: u32 },
    ModifyRefraction { delta: i32 },
    SetPosition { x: i32, y: i32 },
    ModifyReputation { faction: String, delta: i32 },
    GainSaltScrip { amount: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceEffect {
    GainLightEnergy { amount: i32 },
    GainVoidEnergy { amount: i32 },
    GainVoidExposure { amount: i32 },
    GainResonanceEnergy { amount: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapEffect {
    SetTile { x: i32, y: i32, tile_type: String },
    RevealAll,
    SpawnEnemy { id: String, x: i32, y: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestEffect {
    Activate { quest_id: String },
    Complete { quest_id: String },
    Progress { quest_id: String, objective_id: String, amount: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StormEffect {
    EditTile { x: i32, y: i32, edit_type: String },
    AdvanceTimer { delta: i32 },
    SetIntensity { level: u8 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusEffect {
    Apply { target: Target, effect_id: String, duration: i32 },
    Remove { target: Target, effect_id: String },
    Tick { target: Target, effect_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventEffect {
    Log { message: String, msg_type: String },
    EmitEvent { event: String },
    OpenBook { book_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Player,
    Enemy { index: usize },
    EnemyAt { x: i32, y: i32 },
    Npc { index: usize },
}
```

### 2.2 Presentation effects (not traced)

```rust
/// Visual feedback only. Never traced, never tested against.
#[derive(Debug, Clone)]
pub enum Presentation {
    DamageNumber { x: i32, y: i32, amount: i32 },
    HitFlash { target: Target },
    ScreenShake { intensity: f32 },
    ParticleBurst { x: i32, y: i32, particle_type: String },
    LogMessage { text: String, msg_type: String },
}
```

### 2.3 QueryContext

```rust
/// Read-only view of game state for rule functions.
/// Constructed once per command dispatch, borrows shared refs.
pub struct QueryContext<'a> {
    pub player: &'a PlayerState,
    pub world: &'a WorldState,
    pub turn: u32,
    // Spatial indices (rebuilt on mutation)
    pub enemy_positions: &'a HashMap<(i32, i32), usize>,
    pub item_positions: &'a HashMap<(i32, i32), Vec<usize>>,
    pub npc_positions: &'a HashMap<(i32, i32), usize>,
    pub visible: &'a HashSet<usize>,
    // DES mock overrides
    pub mock_combat_hit: Option<bool>,
    pub mock_combat_damage: Option<i32>,
}

impl<'a> QueryContext<'a> {
    pub fn from_state(state: &'a GameState) -> Self {
        QueryContext {
            player: &state.player,
            world: &state.world,
            turn: state.turn,
            enemy_positions: &state.enemy_positions,
            item_positions: &state.item_positions,
            npc_positions: &state.npc_positions,
            visible: &state.visible,
            mock_combat_hit: state.mock_combat_hit,
            mock_combat_damage: state.mock_combat_damage,
        }
    }

    /// Convenience: look up item definition by ID
    pub fn item_def(&self, id: &str) -> Option<&'static ItemDef> {
        get_item_def(id)
    }

    /// Convenience: get enemy by index
    pub fn enemy(&self, idx: usize) -> Option<&'a Enemy> {
        self.world.enemies.get(idx)
    }

    /// Convenience: enemy at position
    pub fn enemy_at(&self, x: i32, y: i32) -> Option<(usize, &'a Enemy)> {
        self.enemy_positions.get(&(x, y))
            .and_then(|&idx| self.world.enemies.get(idx).map(|e| (idx, e)))
    }
}
```

### 2.4 Rule functions

Rules are free functions. No trait, no struct, no dynamic dispatch.

```rust
/// Rule signature. Every rule follows this pattern.
/// Input: command-specific args + shared context + RNG
/// Output: effects to apply (game effects + presentation)
pub struct RuleOutput {
    pub effects: Vec<Effect>,
    pub presentation: Vec<Presentation>,
}

// Example: use_item rule
pub fn rule_use_item(
    item_index: usize,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    // Validation
    let item_id = match ctx.player.inventory.get(item_index) {
        Some(id) => id.clone(),
        None => return RuleOutput { effects, presentation },
    };
    let def = match ctx.item_def(&item_id) {
        Some(d) => d,
        None => return RuleOutput { effects, presentation },
    };
    if !def.usable {
        presentation.push(Presentation::LogMessage {
            text: format!("You can't use {} right now.", def.name),
            msg_type: "info".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // AP cost
    let cost = action_cost("use_item");
    if ctx.player.ap < cost {
        return RuleOutput { effects, presentation };
    }
    effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }));

    // Consume the item
    effects.push(Effect::Item(ItemEffect::Consume {
        item_id: item_id.clone(),
        inventory_index: item_index,
    }));

    // Healing
    if def.heal > 0 {
        let heal = def.heal.min(ctx.player.max_hp - ctx.player.hp);
        effects.push(Effect::Player(PlayerEffect::Heal { amount: heal }));
        presentation.push(Presentation::LogMessage {
            text: format!("You use {}. (+{} HP)", def.name, heal),
            msg_type: "loot".into(),
        });
    }

    // Refraction reduction
    if def.reduces_refraction > 0 {
        let reduce = def.reduces_refraction.min(ctx.player.refraction as i32);
        effects.push(Effect::Player(PlayerEffect::ModifyRefraction { delta: -reduce }));
    }

    // Map reveal
    if def.reveals_map {
        effects.push(Effect::Map(MapEffect::RevealAll));
    }

    // Resource effects
    if def.light_energy > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainLightEnergy {
            amount: def.light_energy,
        }));
    }
    if def.void_energy > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainVoidEnergy {
            amount: def.void_energy,
        }));
    }
    if def.resonance_energy > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainResonanceEnergy {
            amount: def.resonance_energy,
        }));
    }

    // Book opening
    if let Some(book_id) = &def.book_id {
        effects.push(Effect::Event(EventEffect::OpenBook {
            book_id: book_id.clone(),
        }));
    }

    RuleOutput { effects, presentation }
}
```

### 2.5 Trace

```rust
/// A single entry in the trace log.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub turn: u32,
    pub source: TraceSource,
    pub effect: Effect,
}

#[derive(Debug, Clone)]
pub enum TraceSource {
    Rule { name: &'static str },
    Reaction { name: &'static str, trigger: Box<Effect> },
}

/// The trace itself. Only populated during DES runs.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub entries: Vec<TraceEntry>,
    pub enabled: bool,
}

impl Trace {
    pub fn record(&mut self, effect: &Effect, source: TraceSource, turn: u32) {
        if self.enabled {
            self.entries.push(TraceEntry {
                turn,
                source,
                effect: effect.clone(),
            });
        }
    }

    /// Query: did this effect occur?
    pub fn contains(&self, effect: &Effect) -> bool {
        self.entries.iter().any(|e| &e.effect == effect)
    }

    /// Query: all effects from a specific rule
    pub fn from_rule(&self, name: &str) -> Vec<&Effect> {
        self.entries.iter()
            .filter(|e| matches!(&e.source, TraceSource::Rule { name: n } if *n == name))
            .map(|e| &e.effect)
            .collect()
    }

    /// Query: all effects of a specific variant
    pub fn effects_matching<F>(&self, predicate: F) -> Vec<&Effect>
    where F: Fn(&Effect) -> bool {
        self.entries.iter()
            .filter(|e| predicate(&e.effect))
            .map(|e| &e.effect)
            .collect()
    }
}
```

---

## 3. The Game Loop

### 3.1 Dispatch cycle

```
Input → Command → Rule(ctx, rng) → Vec<Effect>
                                      ↓
                              apply(state, effect)  ← mechanical, no logic
                                      ↓
                              reactions(effect, ctx, rng) → Vec<Effect>  ← max depth 10
                                      ↓
                              apply(state, reaction_effects)
                                      ↓
                              derives: update_fov, update_lighting, rebuild_spatial_index
```

### 3.2 Apply function

The apply function is a pure `match` — no logic, no branching, no decisions. It's the only place state mutates.

```rust
impl GameState {
    pub fn apply_effect(&mut self, effect: &Effect) {
        match effect {
            Effect::Player(e) => self.apply_player_effect(e),
            Effect::Combat(e) => self.apply_combat_effect(e),
            Effect::Item(e) => self.apply_item_effect(e),
            Effect::Map(e) => self.apply_map_effect(e),
            Effect::Quest(e) => self.apply_quest_effect(e),
            Effect::Storm(e) => self.apply_storm_effect(e),
            Effect::Status(e) => self.apply_status_effect_v2(e),
            Effect::Resource(e) => self.apply_resource_effect(e),
            Effect::Event(e) => self.apply_event_effect(e),
        }
    }

    fn apply_player_effect(&mut self, effect: &PlayerEffect) {
        match effect {
            PlayerEffect::Heal { amount } => {
                self.player.hp = (self.player.hp + amount).min(self.player.max_hp);
            }
            PlayerEffect::SpendAp { amount } => {
                self.player.ap -= amount;
            }
            PlayerEffect::GainXp { amount } => {
                self.player.xp += amount;
            }
            PlayerEffect::LevelUp { new_level } => {
                self.player.level = *new_level;
            }
            PlayerEffect::ModifyRefraction { delta } => {
                self.player.refraction =
                    (self.player.refraction as i32 + delta).max(0) as u32;
            }
            PlayerEffect::SetPosition { x, y } => {
                self.player.x = *x;
                self.player.y = *y;
            }
            PlayerEffect::ModifyReputation { faction, delta } => {
                *self.player.faction_reputation
                    .entry(faction.clone())
                    .or_insert(0) += delta;
            }
            PlayerEffect::GainSaltScrip { amount } => {
                self.player.salt_scrip += amount;
            }
        }
    }

    // ... similar for each domain. Every arm is a direct field assignment.
    // No function calls, no conditionals, no side effects beyond the assignment.
}
```

### 3.3 Orchestration (in state.rs)

The orchestrator is the thin layer that wires command → rule → apply → reactions → derives:

```rust
impl GameState {
    /// Central dispatch. All commands go through here.
    pub fn dispatch(&mut self, command: Command) {
        let ctx = QueryContext::from_state(self);
        let output = match &command {
            Command::UseItem { index } => rule_use_item(*index, &ctx, &mut self.rng),
            Command::Move { dx, dy } => rule_move(*dx, *dy, &ctx, &mut self.rng),
            Command::Attack { target } => rule_attack(target, &ctx, &mut self.rng),
            // ... one arm per command
            _ => RuleOutput::default(),
        };

        // Apply game effects
        for effect in &output.effects {
            self.apply_effect(effect);
            self.trace.record(effect, TraceSource::Rule { name: command.name() }, self.turn);
        }

        // Apply presentation
        for p in &output.presentation {
            self.apply_presentation(p);
        }

        // Run reactions (max depth 10)
        self.run_reactions(&output.effects, 0);

        // Derives
        self.update_fov();
        self.update_lighting();
    }

    fn run_reactions(&mut self, effects: &[Effect], depth: u32) {
        if depth >= 10 { return; }

        let mut reaction_effects = Vec::new();
        for effect in effects {
            let ctx = QueryContext::from_state(self);
            let reactions = self.collect_reactions(effect, &ctx);
            for re in reactions {
                for e in &re.effects {
                    self.apply_effect(e);
                    self.trace.record(e,
                        TraceSource::Reaction {
                            name: re.source,
                            trigger: Box::new(effect.clone()),
                        },
                        self.turn,
                    );
                    reaction_effects.push(e.clone());
                }
            }
        }

        if !reaction_effects.is_empty() {
            self.run_reactions(&reaction_effects, depth + 1);
        }
    }

    fn collect_reactions(&self, effect: &Effect, ctx: &QueryContext) -> Vec<ReactionOutput> {
        let mut results = Vec::new();
        // Each reaction is a function registered here
        match effect {
            Effect::Combat(CombatEffect::Kill { target }) => {
                results.push(reaction_on_kill(target, ctx, &mut self.rng));
            }
            // Loot drops, quest progress, etc.
            _ => {}
        }
        results
    }
}
```

### 3.4 What this means for existing code

During migration, legacy methods (`self.use_item(idx)`) coexist with `self.dispatch(Command::UseItem { index: idx })`. The DES interpreter can call either. Migration is method-by-method — convert `use_item` to a rule, update the DES dispatch, delete the old method. No flag day.

---

## 4. Verification Strategy

This is the core motivation. Three layers of verification, each catching different failure modes.

### 4.1 Layer 1: Rule unit tests (NEW — does not exist today)

Rule functions are pure: `(args, ctx, rng) → Vec<Effect>`. They can be tested without GameState.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> TestContext {
        // Minimal context builder for tests
        TestContext::new()
            .with_player_hp(100)
            .with_player_ap(10)
            .with_inventory(vec!["healing_salve".into()])
    }

    #[test]
    fn use_healing_item_produces_heal_effect() {
        let ctx = test_ctx().build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_use_item(0, &ctx, &mut rng);

        assert!(output.effects.contains(
            &Effect::Player(PlayerEffect::Heal { amount: 25 })
        ));
        assert!(output.effects.contains(
            &Effect::Player(PlayerEffect::SpendAp { amount: 1 })
        ));
        assert!(output.effects.contains(
            &Effect::Item(ItemEffect::Consume {
                item_id: "healing_salve".into(),
                inventory_index: 0,
            })
        ));
    }

    #[test]
    fn use_item_with_no_ap_produces_nothing() {
        let ctx = test_ctx().with_player_ap(0).build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_use_item(0, &ctx, &mut rng);

        assert!(output.effects.is_empty());
    }
}
```

This is the layer that catches "rule exists but does nothing" — the scaffold-and-abandon pattern. If a rule returns an empty `Vec<Effect>`, the test fails. An AI agent writing a new ability rule must produce effects or the test doesn't pass.

**TestContext builder:** A lightweight struct that constructs a `QueryContext` from minimal setup. No GameState needed. This is what makes rule tests fast and focused.

```rust
pub struct TestContext {
    player: PlayerState,
    world: WorldState,
    // ... minimal fields
}

impl TestContext {
    pub fn new() -> Self { /* defaults */ }
    pub fn with_player_hp(mut self, hp: i32) -> Self { self.player.hp = hp; self }
    pub fn with_player_ap(mut self, ap: i32) -> Self { self.player.ap = ap; self }
    pub fn with_inventory(mut self, items: Vec<String>) -> Self {
        self.player.inventory = items; self
    }
    pub fn with_enemy_at(mut self, id: &str, x: i32, y: i32) -> Self { /* ... */ self }
    pub fn build(&self) -> QueryContext<'_> { QueryContext::from_test(self) }
}
```

### 4.2 Layer 2: DES scenarios (EXISTS — needs extension, not replacement)

The current DES is 2,415 LOC with ~50 assertion types and ~30 action types. This is valuable infrastructure. The DES does NOT need to be rewritten. It needs two additions:

**Addition 1: Effect assertions.** A new assertion type that checks the trace.

```json
{
    "assertions": [
        {
            "type": "effect_occurred",
            "effect": { "type": "player/heal", "amount": 25 },
            "at_end": true
        },
        {
            "type": "effect_not_occurred",
            "effect": { "type": "combat/kill" },
            "at_end": true
        },
        {
            "type": "effect_count",
            "pattern": "player/*",
            "op": "ge",
            "value": 2,
            "at_end": true
        }
    ]
}
```

This is additive — existing assertions (`player_hp`, `enemy_alive`, `inventory_contains`) continue to work unchanged. Effect assertions are a new category alongside them.

**Addition 2: Trace dump on failure.** When a DES scenario fails, dump the trace to the test output. This tells the agent exactly what happened:

```
TRACE for scenario "use_healing_salve":
  Turn 1:
    [rule:use_item] Player(SpendAp { amount: 1 })
    [rule:use_item] Player(Heal { amount: 25 })
    [rule:use_item] Item(Consume { item_id: "healing_salve", inventory_index: 0 })
  Assertion FAILED: player_hp { op: eq, value: 125 }
    Actual: player.hp = 100
```

The agent can now see that the heal effect was produced but the apply function didn't work, or that the rule didn't produce the expected effect. This is the "inspectable record" the architecture promises.

**What stays the same in DES:**
- Scenario format (JSON)
- Action dispatch (Move, Attack, UseItem, Wait, etc.)
- All existing assertion types
- Mocks (combat_always_hit, combat_fixed_damage)
- Scenario inheritance (BASE_*)
- Map setup (clear_radius, ensure_paths)

**What changes in DES:**
- `execute_player_action` calls `self.state.dispatch(command)` instead of `self.state.use_item(idx)` for migrated commands
- Trace is enabled during DES runs (`self.state.trace.enabled = true`)
- New assertion types: `effect_occurred`, `effect_not_occurred`, `effect_count`, `effects_from_rule`
- Trace dump on assertion failure

### 4.3 Layer 3: Compile-time verification (FREE — from Rust's type system)

The `Effect` enum gives us exhaustive match. If a new `CombatEffect::Disarm { ... }` variant is added but not handled in `apply_combat_effect`, the compiler errors. This is the structural gate against scaffold-and-abandon — you can't add an effect variant without handling it.

Similarly, if a new `Command::UseAbility { ... }` variant is added but not handled in `dispatch`, the compiler errors. The agent must wire the command to a rule.

### 4.4 Verification summary

| Failure mode | Caught by | Layer |
|-------------|----------|-------|
| Rule returns wrong effects | Rule unit test | 1 |
| Rule returns empty effects (unwired) | Rule unit test | 1 |
| Apply function doesn't handle effect | Compiler (exhaustive match) | 3 |
| Command not dispatched to rule | Compiler (exhaustive match) | 3 |
| Effect applied but wrong state change | DES scenario (state assertions) | 2 |
| Reaction not triggered | DES scenario (effect assertions) | 2 |
| Full gameplay path broken | DES scenario (integration) | 2 |
| New effect variant not handled | Compiler | 3 |

---

## 5. Migration Plan

### 5.1 Phase 0: Foundation (on main, ~200 LOC)

Create the infrastructure. Zero behavior change.

1. Create `src/game/effects/mod.rs` with `Effect`, `Presentation`, `Target` enums
2. Create `src/game/effects/trace.rs` with `Trace`, `TraceEntry`, `TraceSource`
3. Create `src/game/effects/context.rs` with `QueryContext` and `TestContext`
4. Create `src/game/effects/apply.rs` with `apply_effect` (empty match arms initially)
5. Add `trace: Trace` field to `GameState` (default disabled)
6. Add `pub mod effects;` to `game/mod.rs`
7. Verify: `cargo build`, `cargo test` — nothing changes

**File structure:**
```
src/game/effects/
├── mod.rs          # Effect, Presentation, Target, RuleOutput enums
├── trace.rs        # Trace, TraceEntry, TraceSource
├── context.rs      # QueryContext, TestContext
└── apply.rs        # GameState::apply_effect and domain-specific apply fns
```

### 5.2 Phase 1: use_item extraction (on branch, proof of concept)

Convert `use_item` from a 140-LOC method on GameState to a rule function.

1. Write `src/game/rules/item.rs` with `rule_use_item`
2. Write rule unit tests (TestContext-based, no GameState)
3. Fill in `apply_player_effect`, `apply_item_effect`, `apply_resource_effect` match arms
4. Add `Command::UseItem` to dispatch
5. Update DES `execute_player_action` for `Action::UseItem` to call dispatch
6. Enable trace in DES, add effect assertions to existing item scenarios
7. Delete old `GameState::use_item` method
8. Verify: all existing DES scenarios pass, new rule tests pass

**Retrospective gate:** After Phase 1, evaluate:
- Did QueryContext work ergonomically?
- How many Effect variants were needed? (estimate: ~15 for use_item)
- Did the rule function feel natural to write?
- Could an AI agent write meaningful unit tests?
- Did the trace dump help debug a failing scenario?

If the answer to any of these is "no," stop and reassess before Phase 2.

### 5.3 Phase 2: Movement (on branch)

Convert `MovementSystem::try_move` to a rule.

1. Write `src/game/rules/movement.rs` with `rule_move`
2. Handle the priority chain: NPC interaction → combat (bump attack) → move
3. Fill in `apply_map_effect`, `apply_combat_effect` arms as needed
4. Add movement-triggered reactions (tile effects, item pickup)
5. Update DES dispatch
6. Delete old movement path

### 5.4 Phase 3: Combat (on branch)

Convert `CombatSystem::attack_melee` and `ranged_attack` to rules.

1. Write `src/game/rules/combat.rs`
2. Wire `reaction_on_kill` for loot drops and quest progress
3. This is where reactions prove themselves — kill → loot → quest progress is a 3-step chain

### 5.5 Phase 3.5: Sub-state extraction (on branch)

Informed by Phases 1-3, extract sub-states from GameState:

1. Move combat-related fields to a sub-struct if the pattern is clear
2. Simplify QueryContext if sub-states provide cleaner access
3. This is optional — only do it if Phases 1-3 reveal clear boundaries

### 5.6 Phase 4: end_turn decomposition (on branch)

Convert `end_turn` from a fan-out-of-15 method to a Phase Sequence:

```rust
fn end_turn_sequence() -> Vec<Command> {
    vec![
        Command::TickStatusEffects,
        Command::RunAI,
        Command::TickStorm,
        Command::AdvanceTime,
        Command::UpdateEncounters,
        Command::ProcessEvents,
    ]
}
```

Each sub-command dispatches to its own rule. The sequence is explicit and inspectable.

### 5.7 Phase 5: DES trace integration

1. Add `effect_occurred`, `effect_not_occurred`, `effect_count` assertion types
2. Add trace dump on failure
3. Write migration sentinel scenarios that assert on effects
4. Update existing scenarios to use effect assertions where valuable

---

## 6. What We Dropped from ESCAEV

The ESCAEV v2 proposal had 8 atomics and 10 grammar rules. VERA keeps the useful parts and drops the formalism that doesn't earn its weight:

| ESCAEV concept | Status | Reason |
|---------------|--------|--------|
| State Facet | Kept (implicit) | Just fields on structs — no need to name it |
| Query / QueryContext | Kept | Solves borrow checker ergonomics |
| Command | Kept | Standard Rust enum |
| Effect (domain-scoped) | Kept | Core of the pattern |
| Rule (pure function) | Kept | Core of the pattern |
| Reaction | Kept (simplified) | Functions, not a registry. Match on effects. |
| Derive | Kept (implicit) | Just `update_fov()` etc. — no need to formalize |
| Trace | Kept | Test infrastructure |
| Phase Sequence | Deferred to Phase 4 | Only needed for end_turn |
| Priority Chain | Deferred to Phase 2 | Only needed for movement |
| Sequential Execution | Deferred to Phase 4 | Only needed for AI turns |
| Grammar rules 1-10 | Dropped as formal rules | They're just "rules return effects, apply is mechanical" |
| GameEffect/PresentationEffect naming | Simplified to Effect/Presentation | Less jargon |
| DeferredCommand | Dropped | Reactions return effects directly. Commands are queued if needed. |
| Coarse/fine effect granularity | Dropped | Start with one granularity. Add zoom later if needed. |

The pattern is: **pure functions return effect enums, a mechanical apply mutates state, traces record what happened.** That's VERA. Everything else is implementation detail that emerges during migration.

---

## 7. Considerations

### 7.1 RNG ordering

Rules must consume RNG in the same order as the imperative code they replace. Same seed must produce same gameplay. During migration, run both old and new paths and compare RNG state after each command. If they diverge, the rule has a bug.

### 7.2 Save compatibility

Effects are ephemeral — they're not serialized. The Trace is not saved. GameState fields don't change (same PlayerState, WorldState, NarrativeEngine). Save format is unaffected by the migration.

### 7.3 Performance

Rule functions allocate `Vec<Effect>` per command. For a turn-based game processing ~1 command per player action, this is negligible. If profiling shows allocation pressure (unlikely), effects can use `SmallVec<[Effect; 8]>`.

### 7.4 Renderer threading

The renderer is read-only and orthogonal to this architecture. See `NOTE_RENDERER_THREADING.md`. The effect system doesn't cross the render boundary. Threading the renderer is a separate concern for after Phase 4.
