use crate::game::action::action_cost;
use crate::game::combat::{default_weapon, get_weapon_def, roll_attack, CombatResult};
use crate::game::effects::context::QueryContext;
use crate::game::effects::{
    CombatEffect, Effect, ItemEffect, PlayerEffect, Presentation, RuleOutput,
};
use crate::game::stat_effect::{resolve_stat, resolve_stat_i32, StatEffectSource};
use rand_chacha::ChaCha8Rng;

/// Apply mock overrides to a combat result.
fn apply_mocks(ctx: &QueryContext, mut result: CombatResult) -> CombatResult {
    if let Some(force_hit) = ctx.mock_combat_hit {
        result.hit = force_hit;
        if !force_hit {
            result.damage = 0;
        }
    }
    if let Some(dmg) = ctx.mock_combat_damage
        && result.hit
    {
        result.damage = dmg;
    }
    result
}

/// Pure rule: melee attack against enemy at (target_x, target_y).
///
/// Produces CombatEffect variants for hit/miss/kill, PlayerEffect for AP and XP,
/// and Presentation for log messages. Does NOT handle swarm aggro, reflect damage,
/// split on death, or visual effects — those are post-processed in dispatch.
pub fn rule_melee_attack(
    target_x: i32,
    target_y: i32,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let ei = match ctx.enemy_idx_at(target_x, target_y) {
        Some(i) => i,
        None => return RuleOutput::default(),
    };

    let cost = action_cost("attack_melee");
    if ctx.player.ap < cost {
        return RuleOutput::default();
    }
    effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }));
    effects.push(Effect::Combat(CombatEffect::Provoke { enemy_idx: ei }));

    let weapon = ctx
        .player
        .equipped_weapon
        .as_ref()
        .and_then(|id| get_weapon_def(id))
        .unwrap_or_else(default_weapon);

    let enemy = match ctx.enemy(ei) {
        Some(e) => e,
        None => return RuleOutput { effects, presentation },
    };
    let enemy_stat_effects = enemy.collect_stat_effects();
    let enemy_reflex = crate::game::stat_effect::resolve_stat_i32(enemy.def().map(|d| d.reflex).unwrap_or(0), "reflex", &enemy_stat_effects);
    let enemy_armor = crate::game::stat_effect::resolve_stat_i32(enemy.def().map(|d| d.armor).unwrap_or(0), "armor", &enemy_stat_effects);

    let player_stat_effects = ctx.player.collect_stat_effects();
    let accuracy_bonus = resolve_stat(0.0, "melee_accuracy_bonus", &player_stat_effects);
    let damage_bonus = resolve_stat(0.0, "melee_damage_bonus", &player_stat_effects);
    let cover_bonus = -(accuracy_bonus * 100.0) as i32;

    let result = roll_attack(rng, weapon, enemy_reflex, enemy_armor, cover_bonus);
    let result = apply_mocks(ctx, result);
    let name = enemy.name().to_string();

    if !result.hit {
        effects.push(Effect::Combat(CombatEffect::Miss { enemy_idx: ei }));
        presentation.push(Presentation::LogMessage {
            text: format!("You miss the {}.", name),
            msg_type: "combat".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let mut dmg = result.damage;
    dmg = (dmg as f32 * (1.0 + damage_bonus)) as i32;
    dmg += resolve_stat_i32(0, "damage_bonus", &player_stat_effects);

    effects.push(Effect::Combat(CombatEffect::DealDamage {
        enemy_idx: ei,
        amount: dmg,
    }));
    effects.push(Effect::Player(PlayerEffect::RecordDamageDealt {
        amount: dmg as u32,
    }));

    if enemy.hp - dmg <= 0 {
        effects.push(Effect::Combat(CombatEffect::Kill {
            enemy_idx: ei,
            enemy_id: enemy.id.clone(),
            x: target_x,
            y: target_y,
        }));
        if let Some(def) = enemy.def()
            && def.xp_value > 0
        {
            effects.push(Effect::Player(PlayerEffect::GainXp {
                amount: def.xp_value,
            }));
        }
        presentation.push(Presentation::LogMessage {
            text: format!("You kill the {}!", name),
            msg_type: "combat".into(),
        });
    } else {
        let crit_str = if result.crit { " CRITICAL!" } else { "" };
        presentation.push(Presentation::LogMessage {
            text: format!("You hit the {} for {} damage.{}", name, dmg, crit_str),
            msg_type: "combat".into(),
        });
    }

    RuleOutput { effects, presentation }
}

/// Pure rule: ranged attack against enemy at (target_x, target_y).
pub fn rule_ranged_attack(
    target_x: i32,
    target_y: i32,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let weapon = match ctx
        .player
        .equipped_weapon
        .as_ref()
        .and_then(|id| get_weapon_def(id))
    {
        Some(w) if w.range > 1 => w,
        _ => {
            presentation.push(Presentation::LogMessage {
                text: "No ranged weapon equipped.".into(),
                msg_type: "combat".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    let dist = (target_x - ctx.player.x).abs() + (target_y - ctx.player.y).abs();
    if dist > weapon.range {
        presentation.push(Presentation::LogMessage {
            text: "Target out of range.".into(),
            msg_type: "combat".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let target_idx = ctx.map.idx(target_x, target_y);
    if !ctx.visible.contains(&target_idx) {
        presentation.push(Presentation::LogMessage {
            text: "Can't see target.".into(),
            msg_type: "combat".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let cost = weapon.ap_cost;
    if ctx.player.ap < cost {
        return RuleOutput::default();
    }

    // Consume ammo
    if let Some(ammo_type) = &weapon.ammo_type {
        if let Some(idx) = ctx.player.inventory.iter().position(|id| id == ammo_type) {
            effects.push(Effect::Item(ItemEffect::RemoveFromInventory { index: idx }));
        } else {
            presentation.push(Presentation::LogMessage {
                text: format!("Out of {}.", ammo_type.replace('_', " ")),
                msg_type: "combat".into(),
            });
            return RuleOutput { effects, presentation };
        }
    }

    effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }));

    let ei = match ctx.enemy_idx_at(target_x, target_y) {
        Some(i) => i,
        None => {
            presentation.push(Presentation::LogMessage {
                text: "No target there.".into(),
                msg_type: "combat".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    effects.push(Effect::Combat(CombatEffect::Provoke { enemy_idx: ei }));

    let enemy = match ctx.enemy(ei) {
        Some(e) => e,
        None => return RuleOutput { effects, presentation },
    };
    let enemy_stat_effects = enemy.collect_stat_effects();
    let enemy_reflex = crate::game::stat_effect::resolve_stat_i32(enemy.def().map(|d| d.reflex).unwrap_or(0), "reflex", &enemy_stat_effects);
    let enemy_armor = crate::game::stat_effect::resolve_stat_i32(enemy.def().map(|d| d.armor).unwrap_or(0), "armor", &enemy_stat_effects);

    let player_stat_effects = ctx.player.collect_stat_effects();
    let accuracy_bonus = resolve_stat(0.0, "ranged_accuracy_bonus", &player_stat_effects);
    let damage_bonus = resolve_stat(0.0, "ranged_damage_bonus", &player_stat_effects);
    let cover_bonus = -(accuracy_bonus * 100.0) as i32;

    let result = roll_attack(rng, weapon, enemy_reflex, enemy_armor, cover_bonus);
    let result = apply_mocks(ctx, result);
    let name = enemy.name().to_string();

    if !result.hit {
        effects.push(Effect::Combat(CombatEffect::Miss { enemy_idx: ei }));
        presentation.push(Presentation::LogMessage {
            text: format!("Your shot misses the {}.", name),
            msg_type: "combat".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let mut dmg = result.damage;
    dmg = (dmg as f32 * (1.0 + damage_bonus)) as i32;

    effects.push(Effect::Combat(CombatEffect::DealDamage {
        enemy_idx: ei,
        amount: dmg,
    }));

    if enemy.hp - dmg <= 0 {
        effects.push(Effect::Combat(CombatEffect::Kill {
            enemy_idx: ei,
            enemy_id: enemy.id.clone(),
            x: target_x,
            y: target_y,
        }));
        if let Some(def) = enemy.def()
            && def.xp_value > 0
        {
            effects.push(Effect::Player(PlayerEffect::GainXp {
                amount: def.xp_value,
            }));
        }
        presentation.push(Presentation::LogMessage {
            text: format!("You kill the {} with a ranged shot!", name),
            msg_type: "combat".into(),
        });
    } else {
        let crit_str = if result.crit { " CRITICAL!" } else { "" };
        presentation.push(Presentation::LogMessage {
            text: format!("You hit the {} for {} damage.{}", name, dmg, crit_str),
            msg_type: "combat".into(),
        });
    }

    RuleOutput { effects, presentation }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::effects::context::TestContext;
    use crate::game::enemy::Enemy;
    use rand::SeedableRng;

    fn combat_ctx() -> TestContext {
        TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10)
            .with_enemy(Enemy::new(6, 5, "shard_spider"))
            .with_mock_combat_hit(true)
            .with_mock_combat_damage(3)
    }

    #[test]
    fn melee_hit_produces_damage_and_ap() {
        let tc = combat_ctx();
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_melee_attack(6, 5, &ctx, &mut rng);

        assert!(output.effects.contains(&Effect::Player(PlayerEffect::SpendAp {
            amount: action_cost("attack_melee")
        })));
        assert!(output.effects.contains(&Effect::Combat(CombatEffect::DealDamage {
            enemy_idx: 0,
            amount: 3,
        })));
        assert!(output.effects.contains(&Effect::Combat(CombatEffect::Provoke {
            enemy_idx: 0,
        })));
    }

    #[test]
    fn melee_miss_produces_miss_effect() {
        let tc = combat_ctx().with_mock_combat_hit(false);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_melee_attack(6, 5, &ctx, &mut rng);

        assert!(output.effects.contains(&Effect::Combat(CombatEffect::Miss {
            enemy_idx: 0,
        })));
        assert!(!output.effects.iter().any(|e| matches!(e, Effect::Combat(CombatEffect::DealDamage { .. }))));
    }

    #[test]
    fn melee_kill_produces_kill_effect() {
        // Enemy has 5 HP, mock damage 10 → kill
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10)
            .with_enemy(Enemy::new(6, 5, "shard_spider"))
            .with_mock_combat_hit(true)
            .with_mock_combat_damage(10);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_melee_attack(6, 5, &ctx, &mut rng);

        assert!(output.effects.iter().any(|e| matches!(
            e,
            Effect::Combat(CombatEffect::Kill { enemy_idx: 0, .. })
        )));
    }

    #[test]
    fn melee_no_ap_produces_nothing() {
        let tc = combat_ctx().with_player_ap(0);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_melee_attack(6, 5, &ctx, &mut rng);

        assert!(output.effects.is_empty());
    }

    #[test]
    fn melee_no_enemy_produces_nothing() {
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_melee_attack(6, 5, &ctx, &mut rng);

        assert!(output.effects.is_empty());
    }

    #[test]
    fn mock_combat_damage_overrides_roll() {
        let tc = combat_ctx().with_mock_combat_damage(7);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_melee_attack(6, 5, &ctx, &mut rng);

        assert!(output.effects.contains(&Effect::Combat(CombatEffect::DealDamage {
            enemy_idx: 0,
            amount: 7,
        })));
    }

    #[test]
    fn ranged_no_weapon_produces_log() {
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10)
            .with_enemy(Enemy::new(8, 5, "shard_spider"));
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_ranged_attack(8, 5, &ctx, &mut rng);

        assert!(output.effects.is_empty());
        assert!(!output.presentation.is_empty());
    }
}
