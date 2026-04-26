use super::super::effects::{Effect, ItemEffect, PlayerEffect, Presentation, RuleOutput};
use super::super::effects::context::QueryContext;

/// Pure rule: craft an item from a recipe.
/// Caller must verify station adjacency before calling.
pub fn rule_craft(recipe_id: &str, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let recipe = match super::super::crafting::get_recipe(recipe_id) {
        Some(r) => r,
        None => {
            presentation.push(Presentation::LogMessage {
                text: format!("Unknown recipe: {}", recipe_id),
                msg_type: "warning".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    // salt_sense: reduce each ingredient requirement by 1 (minimum 1)
    let ingredient_reduction: u32 = ctx.player_adaptations.iter()
        .filter_map(|a| a.def())
        .flat_map(|d| d.effects.iter())
        .filter(|e| e.effect_type == "craft_ingredient_reduction")
        .filter_map(|e| e.value)
        .map(|v| v as u32)
        .sum();

    // Check materials with reduction applied
    let can_craft = recipe.materials.iter().all(|(item_id, &count)| {
        let effective = count.saturating_sub(ingredient_reduction).max(1);
        ctx.player.inventory.iter().filter(|id| *id == item_id).count() as u32 >= effective
    });
    if !can_craft {
        presentation.push(Presentation::LogMessage {
            text: "You don't have the required materials.".into(),
            msg_type: "warning".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // Consume materials — track removed indices to avoid double-counting
    let mut removed_indices: Vec<usize> = Vec::new();
    for (item_id, &count) in &recipe.materials {
        let effective_count = count.saturating_sub(ingredient_reduction).max(1);
        for _ in 0..effective_count {
            if let Some(i) = ctx.player.inventory.iter().enumerate()
                .filter(|(idx, _)| !removed_indices.contains(idx))
                .find(|(_, id)| *id == item_id)
                .map(|(idx, _)| idx)
            {
                removed_indices.push(i);
            }
        }
    }
    // Sort descending so higher indices are removed first (no shifting issues)
    removed_indices.sort_unstable_by(|a, b| b.cmp(a));
    for i in removed_indices {
        effects.push(Effect::Item(ItemEffect::RemoveFromInventory { index: i }));
    }

    // Produce output
    for _ in 0..recipe.output_count {
        effects.push(Effect::Item(ItemEffect::AddToInventory { item_id: recipe.output.clone() }));
    }

    presentation.push(Presentation::LogMessage {
        text: format!("Crafted {}.", recipe.name),
        msg_type: "loot".into(),
    });

    RuleOutput { effects, presentation }
}

/// Pure rule: buy an item from a trader NPC.
pub fn rule_buy_item(item_id: &str, npc_id: &str, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let npc_def = match super::super::npc::get_npc_def(npc_id) {
        Some(d) => d,
        None => {
            presentation.push(Presentation::LogMessage {
                text: format!("NPC '{}' not found.", npc_id),
                msg_type: "warning".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    if !npc_def.shop_inventory.contains(&item_id.to_string()) {
        presentation.push(Presentation::LogMessage {
            text: format!("{} doesn't sell that item.", npc_def.name),
            msg_type: "warning".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let item_def = match ctx.item_def(item_id) {
        Some(d) => d,
        None => {
            presentation.push(Presentation::LogMessage {
                text: format!("Item '{}' not found.", item_id),
                msg_type: "warning".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    let price = item_def.value;
    if ctx.player.salt_scrip < price {
        presentation.push(Presentation::LogMessage {
            text: format!("Not enough salt scrip (need {}, have {}).", price, ctx.player.salt_scrip),
            msg_type: "warning".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // Deduct cost (negative gain)
    effects.push(Effect::Player(PlayerEffect::GainSaltScrip {
        amount: 0u32.wrapping_sub(price),
    }));
    effects.push(Effect::Item(ItemEffect::AddToInventory { item_id: item_id.to_string() }));

    presentation.push(Presentation::LogMessage {
        text: format!("Bought {} for {} salt scrip.", item_def.name, price),
        msg_type: "loot".into(),
    });

    RuleOutput { effects, presentation }
}

/// Pure rule: sell an item from inventory by index.
pub fn rule_sell_item(inv_idx: usize, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let item_id = match ctx.player.inventory.get(inv_idx) {
        Some(id) => id.clone(),
        None => {
            presentation.push(Presentation::LogMessage {
                text: "You don't have that item.".into(),
                msg_type: "warning".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    let item_def = match ctx.item_def(&item_id) {
        Some(d) => d,
        None => {
            presentation.push(Presentation::LogMessage {
                text: format!("Item '{}' not found.", item_id),
                msg_type: "warning".into(),
            });
            return RuleOutput { effects, presentation };
        }
    };

    let sell_price = item_def.value / 2;

    effects.push(Effect::Item(ItemEffect::RemoveFromInventory { index: inv_idx }));
    effects.push(Effect::Player(PlayerEffect::GainSaltScrip { amount: sell_price }));

    presentation.push(Presentation::LogMessage {
        text: format!("Sold {} for {} salt scrip.", item_def.name, sell_price),
        msg_type: "loot".into(),
    });

    RuleOutput { effects, presentation }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::effects::context::TestContext;

    #[test]
    fn sell_item_unknown_index_produces_warning() {
        let tc = TestContext::new();
        let ctx = tc.build();
        let output = rule_sell_item(0, &ctx);
        assert!(output.effects.is_empty());
        assert!(!output.presentation.is_empty());
    }

    #[test]
    fn buy_item_unknown_npc_produces_warning() {
        let tc = TestContext::new();
        let ctx = tc.build();
        let output = rule_buy_item("healing_salve", "nonexistent_npc", &ctx);
        assert!(output.effects.is_empty());
        assert!(!output.presentation.is_empty());
    }

    #[test]
    fn craft_unknown_recipe_produces_warning() {
        let tc = TestContext::new();
        let ctx = tc.build();
        let output = rule_craft("nonexistent_recipe", &ctx);
        assert!(output.effects.is_empty());
        assert!(!output.presentation.is_empty());
    }
}
