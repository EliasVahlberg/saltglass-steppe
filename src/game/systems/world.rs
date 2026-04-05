use crate::game::mutations::Mutation;

pub fn handle_world_move(wx: usize, wy: usize) -> Vec<Mutation> {
    vec![Mutation::WorldMove { wx, wy }]
}

pub fn handle_world_move_safe(wx: usize, wy: usize) -> Vec<Mutation> {
    vec![Mutation::WorldMoveSafe { wx, wy }]
}

pub fn handle_follow_path() -> Vec<Mutation> {
    vec![Mutation::FollowWorldPath]
}

pub fn handle_calculate_path(target: (usize, usize)) -> Vec<Mutation> {
    vec![Mutation::CalculateWorldPath { target }]
}

pub fn handle_enter_subterranean() -> Vec<Mutation> {
    vec![Mutation::EnterSubterranean]
}

pub fn handle_exit_subterranean() -> Vec<Mutation> {
    vec![Mutation::ExitSubterranean]
}
