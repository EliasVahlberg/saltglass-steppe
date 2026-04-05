use crate::game::{
    effects::{Presentation, RuleOutput},
    mutations::Mutation,
    rules::actions::{ExamineTarget, InteractTarget, rule_examine, rule_interact},
    state::{GameState, MsgType},
};

pub fn handle_interact(x: i32, y: i32, state: &mut GameState) -> Vec<Mutation> {
    state.ensure_spatial_index();

    let target = if let Some(&idx) = state.spatial.interactable_positions.get(&(x, y))
        && let Some(interactable) = state.world.interactables.get_mut(idx)
    {
        let id = interactable.id.clone();
        let message = interactable.interact();
        InteractTarget::Interactable { id, message }
    } else if let Some(&idx) = state.spatial.npc_positions.get(&(x, y))
        && let Some(npc) = state.world.npcs.get(idx)
    {
        InteractTarget::Npc { id: npc.id.clone(), name: npc.name().to_string() }
    } else if let Some(&idx) = state.spatial.chest_positions.get(&(x, y))
        && let Some(chest) = state.world.chests.get(idx)
    {
        InteractTarget::Chest { name: chest.name().to_string() }
    } else {
        InteractTarget::Nothing
    };

    state.spatial.dirty = true;
    rule_output_to_mutations(rule_interact(target))
}

pub fn handle_examine(x: i32, y: i32, state: &GameState) -> Vec<Mutation> {
    use crate::game::map::Tile;

    let target = if let Some(&idx) = state.spatial.interactable_positions.get(&(x, y))
        && let Some(interactable) = state.world.interactables.get(idx)
    {
        let id = interactable.id.clone();
        let message = interactable.examine();
        ExamineTarget::Interactable { id, message }
    } else if let Some(&idx) = state.spatial.enemy_positions.get(&(x, y))
        && let Some(enemy) = state.world.enemies.get(idx)
        && enemy.hp > 0
    {
        let max_hp = enemy.def().map(|d| d.max_hp).unwrap_or(enemy.hp);
        ExamineTarget::Enemy { name: enemy.name().to_string(), hp: enemy.hp, max_hp }
    } else if let Some(&idx) = state.spatial.npc_positions.get(&(x, y))
        && let Some(npc) = state.world.npcs.get(idx)
    {
        ExamineTarget::Npc { name: npc.name().to_string(), description: npc.description().to_string() }
    } else if let Some(indices) = state.spatial.item_positions.get(&(x, y))
        && !indices.is_empty()
        && let Some(item) = state.world.items.get(indices[0])
    {
        ExamineTarget::Item { name: item.name().to_string() }
    } else if let Some(&idx) = state.spatial.chest_positions.get(&(x, y))
        && let Some(chest) = state.world.chests.get(idx)
    {
        ExamineTarget::Chest { name: chest.name().to_string(), description: chest.description().to_string() }
    } else {
        let tile = state.world.map.get_tile(x, y);
        let desc: &'static str = match tile {
            Tile::Wall { .. } => "A solid wall.",
            Tile::Floor { .. } => "The ground here is clear.",
            Tile::Glass => "Dangerous glass terrain that refracts light.",
            _ => "You examine the area.",
        };
        ExamineTarget::Tile { description: desc }
    };

    rule_output_to_mutations(rule_examine(target))
}

fn rule_output_to_mutations(output: RuleOutput) -> Vec<Mutation> {
    use crate::game::effects::{Effect, EventEffect};
    let mut out = Vec::new();
    for effect in output.effects {
        if let Effect::Event(EventEffect::QuestNotify { kind }) = effect {
            out.push(Mutation::QuestNotify(kind));
        }
    }
    for p in output.presentation {
        let Presentation::LogMessage { text, msg_type } = p;
        let mt = msg_type_from_str(&msg_type);
        out.push(Mutation::LogMessage { text, msg_type: mt });
    }
    out
}

fn msg_type_from_str(s: &str) -> MsgType {
    match s {
        "combat" => MsgType::Combat,
        "loot" => MsgType::Loot,
        "status" => MsgType::Status,
        "warning" => MsgType::Warning,
        "dialogue" => MsgType::Dialogue,
        "social" => MsgType::Social,
        _ => MsgType::System,
    }
}
