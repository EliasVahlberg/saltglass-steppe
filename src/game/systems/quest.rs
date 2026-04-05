use crate::game::{
    mutations::Mutation,
    quest::get_quest_def,
    state::{GameState, MsgType},
};

pub fn handle_accept_quest(quest_id: &str, state: &GameState) -> Vec<Mutation> {
    if !state.player.quest_log.is_quest_available(quest_id, state) {
        return vec![];
    }

    let mut out = vec![Mutation::AcceptQuest(quest_id.to_string())];

    if let Some(def) = get_quest_def(quest_id) {
        out.push(Mutation::LogMessage {
            text: format!("Quest accepted: {}", def.name),
            msg_type: MsgType::System,
        });
        if def.category == "main" && quest_id.starts_with("faction_choice_") {
            let faction = if quest_id.contains("monks") { "Mirror Monks" }
                else if quest_id.contains("engineers") { "Sand-Engineers" }
                else if quest_id.contains("glassborn") { "Glassborn" }
                else { "" };
            if !faction.is_empty() {
                out.push(Mutation::SetFactionAlignment(faction.to_string()));
                out.push(Mutation::LogMessage {
                    text: format!("You have aligned with the {}", faction),
                    msg_type: MsgType::System,
                });
            }
        }
    }
    out
}

pub fn handle_complete_quest(quest_id: &str, state: &GameState) -> Vec<Mutation> {
    // Check completable before mutating
    if !state.player.quest_log.active.iter().any(|q| q.quest_id == quest_id) {
        return vec![];
    }

    let mut out = vec![Mutation::CompleteQuest(quest_id.to_string())];

    if let Some(def) = get_quest_def(quest_id) {
        out.push(Mutation::LogMessage {
            text: format!("Quest completed: {}", def.name),
            msg_type: MsgType::System,
        });
        let reward = &def.reward;
        if reward.xp > 0 {
            // System computes final XP + level-up mutations
            out.extend(xp_mutations(
                state.player.xp, state.player.level,
                state.player.pending_stat_points, state.player.skills.skill_points,
                reward.xp,
            ));
        }
        if reward.salt_scrip > 0 {
            out.push(Mutation::SetPlayerSaltScrip(state.player.salt_scrip + reward.salt_scrip));
            out.push(Mutation::LogMessage {
                text: format!("Received {} salt scrip", reward.salt_scrip),
                msg_type: MsgType::Loot,
            });
        }
        for item_id in &reward.items {
            out.push(Mutation::AddToInventory(item_id.clone()));
        }
        for (faction_id, delta) in &reward.reputation_rewards {
            let current = state.player.faction_reputation.get(faction_id.as_str()).copied().unwrap_or(0);
            out.push(Mutation::SetReputation { faction: faction_id.clone(), value: current + delta });
        }
        for unlocked_id in &reward.unlocks_quests {
            if let Some(unlocked_def) = get_quest_def(unlocked_id) {
                out.push(Mutation::LogMessage {
                    text: format!("New quest available: {}", unlocked_def.name),
                    msg_type: MsgType::System,
                });
            }
        }
    }
    out
}

fn xp_mutations(current_xp: u32, current_level: u32, current_stat_pts: i32,
                current_skill_pts: u32, gain: u32) -> Vec<Mutation> {
    use crate::game::progression::{max_level, stat_points_per_level, xp_for_level};
    let mut out = Vec::new();
    let new_xp = current_xp + gain;
    out.push(Mutation::SetPlayerXp(new_xp));
    out.push(Mutation::LogMessage { text: format!("+{} XP", gain), msg_type: MsgType::System });
    let mut level = current_level;
    let mut stat_pts = current_stat_pts;
    let mut skill_pts = current_skill_pts;
    while level < max_level() && new_xp >= xp_for_level(level + 1) {
        level += 1;
        let pts = stat_points_per_level();
        stat_pts += pts;
        skill_pts += 2;
        out.push(Mutation::SetPlayerLevel(level));
        out.push(Mutation::SetPlayerStatPoints(stat_pts));
        out.push(Mutation::SetPlayerSkillPoints(skill_pts));
        out.push(Mutation::LogMessage {
            text: format!("⬆ LEVEL {}! (+{} stat points, +2 skill points)", level, pts),
            msg_type: MsgType::System,
        });
    }
    out
}
