//! VERA rule functions for Batch A simple player actions.

use super::super::effects::{
    CombatEffect, Effect, ItemEffect, PlayerEffect, Presentation, RuleOutput,
};
use super::super::effects::context::QueryContext;

/// Wait in place. Checks for nearby enemies, increments wait counter,
/// auto-heals after 10 consecutive waits with no enemies nearby.
pub fn rule_wait(ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let enemies_nearby = ctx.enemies.iter().any(|e| {
        if e.hp <= 0 { return false; }
        let dx = (e.x - ctx.player.x).abs();
        let dy = (e.y - ctx.player.y).abs();
        dx <= 8 && dy <= 8
    });

    if enemies_nearby {
        effects.push(Effect::Player(PlayerEffect::ResetWaitCounter));
    } else {
        effects.push(Effect::Player(PlayerEffect::IncrementWaitCounter));
        // Auto-rest after 10 consecutive waits
        if ctx.wait_counter >= 9 && ctx.player.hp < ctx.player.max_hp {
            let heal = (ctx.player.max_hp / 20).max(1);
            effects.push(Effect::Player(PlayerEffect::Heal { amount: heal }));
            effects.push(Effect::Player(PlayerEffect::ResetWaitCounter));
            presentation.push(Presentation::LogMessage {
                text: format!("You rest and recover {} HP.", heal),
                msg_type: "status".into(),
            });
        }
    }

    RuleOutput { effects, presentation }
}

/// Rest to recover 50% max HP. Returns Err presentation if enemies nearby.
pub fn rule_rest(ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    // Check for nearby enemies
    let fov_range = 12; // FOV_RANGE constant
    let enemies_nearby = ctx.enemies.iter().any(|e| {
        let dx = (e.x - ctx.player.x).abs();
        let dy = (e.y - ctx.player.y).abs();
        dx <= fov_range && dy <= fov_range
    });

    if enemies_nearby {
        presentation.push(Presentation::LogMessage {
            text: "You cannot rest with enemies nearby!".into(),
            msg_type: "warning".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let heal_amount = (ctx.player.max_hp as f32 * 0.5) as i32;
    let actual_heal = heal_amount.min(ctx.player.max_hp - ctx.player.hp);

    if actual_heal > 0 {
        effects.push(Effect::Player(PlayerEffect::Heal { amount: actual_heal }));
        presentation.push(Presentation::LogMessage {
            text: format!("You rest and recover {} HP.", actual_heal),
            msg_type: "status".into(),
        });
    } else {
        presentation.push(Presentation::LogMessage {
            text: "You rest but are already at full health.".into(),
            msg_type: "status".into(),
        });
    }

    // Advance 10 turns
    for _ in 0..10 {
        effects.push(Effect::Player(PlayerEffect::AdvanceTurn));
    }

    RuleOutput { effects, presentation }
}

/// Equip an item from inventory to a slot.
pub fn rule_equip(inv_idx: usize, slot: &str, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();

    if inv_idx >= ctx.player.inventory.len() {
        return RuleOutput { effects, presentation: Vec::new() };
    }

    let item_id = ctx.player.inventory[inv_idx].clone();
    effects.push(Effect::Item(ItemEffect::RemoveFromInventory { index: inv_idx }));
    effects.push(Effect::Item(ItemEffect::Equip { item_id, slot: slot.to_string() }));
    effects.push(Effect::Item(ItemEffect::RecalcStats));

    RuleOutput { effects, presentation: Vec::new() }
}

/// Unequip item from slot back to inventory.
pub fn rule_unequip(slot: &str) -> RuleOutput {
    RuleOutput {
        effects: vec![
            Effect::Item(ItemEffect::Unequip { slot: slot.to_string() }),
            Effect::Item(ItemEffect::RecalcStats),
        ],
        presentation: Vec::new(),
    }
}

/// Allocate a stat point.
pub fn rule_allocate_stat(stat: &str, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    if ctx.player.pending_stat_points <= 0 {
        return RuleOutput { effects, presentation };
    }

    match stat {
        "max_hp" | "max_ap" | "reflex" => {
            effects.push(Effect::Player(PlayerEffect::AllocateStat { stat: stat.to_string() }));
            presentation.push(Presentation::LogMessage {
                text: format!("+1 {}", stat),
                msg_type: "status".into(),
            });
        }
        _ => {}
    }

    RuleOutput { effects, presentation }
}

/// Use a psychic ability. Produces effects for the 3 working abilities.
/// Dispatch handles cooldown/energy check via use_ability().
pub fn rule_use_psychic(ability_id: &str, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    presentation.push(Presentation::LogMessage {
        text: format!("You use {}.", ability_id),
        msg_type: "combat".into(),
    });

    match ability_id {
        "stun_aoe" => {
            let mut stunned = 0;
            for (i, enemy) in ctx.enemies.iter().enumerate() {
                let dist = ((enemy.x - ctx.player.x).pow(2)
                    + (enemy.y - ctx.player.y).pow(2)) as f32;
                if dist <= 25.0 {
                    effects.push(Effect::Combat(CombatEffect::StunEnemy {
                        enemy_idx: i,
                        duration: 2,
                    }));
                    stunned += 1;
                }
            }
            presentation.push(Presentation::LogMessage {
                text: format!("Stunned {} enemies.", stunned),
                msg_type: "combat".into(),
            });
        }
        "guaranteed_hit" => {
            effects.push(Effect::Player(PlayerEffect::ApplyStatusEffect {
                effect_id: "guaranteed_hit".into(),
                duration: 1,
            }));
        }
        "phasing" => {
            effects.push(Effect::Player(PlayerEffect::ApplyStatusEffect {
                effect_id: "phasing".into(),
                duration: 5,
            }));
            effects.push(Effect::Player(PlayerEffect::SetPhaseMode { enabled: true }));
        }
        _ => {
            presentation.clear();
            presentation.push(Presentation::LogMessage {
                text: "Effect not implemented.".into(),
                msg_type: "system".into(),
            });
        }
    }

    RuleOutput { effects, presentation }
}

// ---------------------------------------------------------------------------
// Rule unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::effects::context::TestContext;

    #[test]
    fn wait_increments_counter_no_enemies() {
        let tc = TestContext::new().with_player_position(5, 5);
        let ctx = tc.build();
        let output = rule_wait(&ctx);
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::IncrementWaitCounter)));
    }

    #[test]
    fn wait_resets_counter_enemies_nearby() {
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_enemy(crate::game::enemy::Enemy::new(8, 8, "salt_crawler"));
        let ctx = tc.build();
        let output = rule_wait(&ctx);
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::ResetWaitCounter)));
    }

    #[test]
    fn rest_heals_50_percent() {
        let tc = TestContext::new()
            .with_player_hp(50)
            .with_player_max_hp(100);
        let ctx = tc.build();
        let output = rule_rest(&ctx);
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::Heal { amount: 50 })));
        let turn_count = output.effects.iter()
            .filter(|e| matches!(e, Effect::Player(PlayerEffect::AdvanceTurn)))
            .count();
        assert_eq!(turn_count, 10);
    }

    #[test]
    fn rest_blocked_by_enemies() {
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_enemy(crate::game::enemy::Enemy::new(8, 8, "salt_crawler"));
        let ctx = tc.build();
        let output = rule_rest(&ctx);
        assert!(output.effects.is_empty());
    }

    #[test]
    fn equip_produces_effects() {
        let tc = TestContext::new()
            .with_inventory(vec!["glass_shard".into()]);
        let ctx = tc.build();
        let output = rule_equip(0, "weapon", &ctx);
        assert_eq!(output.effects.len(), 3);
    }

    #[test]
    fn allocate_stat_no_points() {
        let tc = TestContext::new();
        let ctx = tc.build();
        let output = rule_allocate_stat("max_hp", &ctx);
        assert!(output.effects.is_empty());
    }

    #[test]
    fn psychic_stun_aoe_stuns_nearby() {
        let mut tc = TestContext::new().with_player_position(5, 5);
        tc.enemies.push(crate::game::enemy::Enemy::new(7, 7, "salt_crawler"));
        tc.enemy_positions.insert((7, 7), 0);
        let ctx = tc.build();
        let output = rule_use_psychic("stun_aoe", &ctx);
        assert!(output.effects.iter().any(|e| matches!(e, Effect::Combat(CombatEffect::StunEnemy { .. }))));
    }
}
