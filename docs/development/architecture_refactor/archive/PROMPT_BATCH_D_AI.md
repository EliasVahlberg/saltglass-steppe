# VERA Batch D Prompt: AI System

**Execute VERA Batch D: AI system migration. Work directly on main.**

Read `docs/development/architecture_refactor/VERA_FULL_MIGRATION.md` — this is Batch D.

**Goal:** Convert `update_enemies()` from a direct-mutation system call to rule-based dispatch where each enemy's turn produces traced effects.

**Current implementation:**

`update_enemies()` in `combat_actions.rs` iterates all enemies, calls the `AiBehavior` trait dispatch (StandardMelee, RangedOnly, Healer, SuicideBomber), which reads world state and mutates enemy positions / triggers attacks directly.

The AI behavior registry in `systems/ai.rs` uses a `BEHAVIOR_REGISTRY` with 4 strategies implementing `AiBehavior::act()`. Each strategy takes `&mut GameState` and an enemy index.

**Migration approach:**

1. **Per-enemy rule function:** `rule_enemy_turn(enemy_idx: usize, ctx: &QueryContext, rng: &mut ChaCha8Rng) -> RuleOutput`

   The rule reads the enemy's behavior type, position, HP, and the player's position from QueryContext. It decides: move toward player, attack, flee, heal, or idle. It returns effects describing the action.

2. **New Effect variants:**
   - `AiEffect::EnemyMove { enemy_idx, new_x, new_y }` — enemy changes position
   - `AiEffect::EnemyAttack { enemy_idx, target_x, target_y, damage }` — enemy attacks player
   - `AiEffect::EnemyHeal { enemy_idx, amount }` — healer behavior
   - `AiEffect::EnemyExplode { enemy_idx, damage, radius }` — suicide bomber
   - `AiEffect::EnemyFlee { enemy_idx, new_x, new_y }` — flee behavior
   - `AiEffect::EnemyIdle { enemy_idx }` — no action (for trace completeness)

3. **Coarse tracing option:** If per-action variants are too noisy, use a single `AiEffect::EnemyActed { enemy_idx, action: AiAction }` where `AiAction` is an enum. The trace shows one entry per enemy per turn. This is sufficient for verification — DES can assert "enemy at (x,y) moved" or "enemy attacked player."

4. **Sequential execution:** Enemies act in index order (0..N). Each enemy's effects are applied before the next enemy acts — this preserves the current behavior where enemy 0's movement affects enemy 1's pathfinding. The dispatch loop:
   ```rust
   TurnPhase::RunAI => {
       for idx in 0..self.world.enemies.len() {
           let ctx = QueryContext::from_fields(self);
           let output = rule_enemy_turn(idx, &ctx, &mut self.rng);
           self.apply_and_trace(output, "rule_enemy_turn");
       }
   }
   ```

5. **QueryContext expansion:** Rules need to read enemy stats, behavior type, demeanor, and flee thresholds. Add `enemy(idx) -> &Enemy` convenience method if not already present.

**Key constraints:**
- RNG ordering: enemies consume RNG for pathfinding and attack rolls in index order. Preserve this.
- The `AiBehavior` trait and `BEHAVIOR_REGISTRY` can stay as decision logic — the rule calls the behavior's decision function but returns effects instead of mutating state.
- Enemy death during AI turns (e.g., suicide bomber kills itself) should produce `CombatEffect::Kill` which can trigger reactions (loot, quest progress).
- Spatial index must be updated between enemy actions if enemies move (an enemy moving affects pathfinding for subsequent enemies).

**New file:** `src/game/rules/ai.rs`

**Tests:**
- Rule unit tests: enemy adjacent to player produces attack effect, enemy far from player produces move effect, healer below threshold produces heal effect, flee behavior when HP low
- All existing DES scenarios must pass — several test enemy behavior (combat_behaviors_test, enemy_ranged_behavior_test, enemy_spawner_behavior_test)

**After completing:** Report whether the `AiBehavior` trait was preserved or replaced, how many AiEffect variants were needed, and whether spatial index updates between enemy actions caused any issues.
