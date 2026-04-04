/// Quest progression is now handled directly via VERA reactions
/// (collect_reactions → QuestNotify effects) and inline quest_log calls.
/// This module is retained for the QuestSystem type re-export.
pub struct QuestSystem;
