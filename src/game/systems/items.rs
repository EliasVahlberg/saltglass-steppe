use crate::game::{
    effects::{Effect, ItemEffect, Presentation, PlayerEffect, RuleOutput},
    mutations::Mutation,
    rules::economy::{rule_buy_item, rule_craft, rule_sell_item},
    state::{GameState, MsgType},
};

pub fn handle_use_item(item_index: usize, ctx: &crate::game::effects::context::QueryContext) -> Vec<Mutation> {
    use crate::game::rules::item::rule_use_item;
    crate::game::systems::rule_output_to_mutations(rule_use_item(item_index, ctx), msg_type_from_str)
}

pub fn handle_use_item_on_tile(item_index: usize, x: i32, y: i32,
                                ctx: &crate::game::effects::context::QueryContext) -> Vec<Mutation> {
    use crate::game::rules::item::rule_use_item_on_tile;
    crate::game::systems::rule_output_to_mutations(rule_use_item_on_tile(item_index, x, y, ctx), msg_type_from_str)
}

pub fn handle_craft(recipe_id: &str, state: &mut GameState) -> Vec<Mutation> {
    // Station check needs spatial index
    state.ensure_spatial_index();
    if let Some(recipe) = crate::game::crafting::get_recipe(recipe_id)
        && let Some(ref station) = recipe.station_required
    {
        let has_station = (-1i32..=1).any(|dx| (-1i32..=1).any(|dy| {
            let pos = (state.player.x + dx, state.player.y + dy);
            state.spatial.interactable_positions.get(&pos)
                .and_then(|&idx| state.world.interactables.get(idx))
                .map(|i| &i.id == station)
                .unwrap_or(false)
        }));
        if !has_station {
            return vec![Mutation::LogMessage {
                text: format!("Requires a nearby {}.", station.replace('_', " ")),
                msg_type: MsgType::Warning,
            }];
        }
    }

    let ctx = crate::game::effects::context::QueryContext::from_state(state);
    rule_output_to_mutations(rule_craft(recipe_id, &ctx))
}

pub fn handle_buy_item(item_id: &str, npc_id: &str, state: &GameState) -> Vec<Mutation> {
    let ctx = crate::game::effects::context::QueryContext::from_state(state);
    rule_output_to_mutations(rule_buy_item(item_id, npc_id, &ctx))
}

pub fn handle_sell_item(item_id: &str, state: &GameState) -> Vec<Mutation> {
    let inv_idx = match state.player.inventory.iter().position(|id| id == item_id) {
        Some(i) => i,
        None => return vec![Mutation::LogMessage {
            text: "You don't have that item.".into(),
            msg_type: MsgType::Warning,
        }],
    };
    let ctx = crate::game::effects::context::QueryContext::from_state(state);
    rule_output_to_mutations(rule_sell_item(inv_idx, &ctx))
}

fn rule_output_to_mutations(output: RuleOutput) -> Vec<Mutation> {
    let mut out = Vec::new();
    for effect in output.effects {
        match effect {
            Effect::Item(ItemEffect::AddToInventory { item_id }) => {
                out.push(Mutation::AddToInventory(item_id));
            }
            Effect::Item(ItemEffect::RemoveFromInventory { index }) => {
                out.push(Mutation::RemoveFromInventory(index));
            }
            Effect::Item(ItemEffect::Consume { inventory_index, .. }) => {
                out.push(Mutation::RemoveFromInventory(inventory_index));
            }
            Effect::Player(PlayerEffect::GainSaltScrip { amount }) => {
                // Can't read current salt_scrip here — emit a delta mutation
                out.push(Mutation::AddSaltScrip(amount));
            }
            Effect::Player(PlayerEffect::SpendAp { amount }) => {
                out.push(Mutation::SpendAp(amount));
            }
            _ => {}
        }
    }
    for p in output.presentation {
        let Presentation::LogMessage { text, msg_type } = p;
        out.push(Mutation::LogMessage { text, msg_type: msg_type_from_str(&msg_type) });
    }
    out
}

fn msg_type_from_str(s: &str) -> MsgType {
    match s {
        "combat" => MsgType::Combat,
        "loot" => MsgType::Loot,
        "status" => MsgType::Status,
        "warning" => MsgType::Warning,
        _ => MsgType::System,
    }
}
