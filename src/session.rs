//! Game session loop, shared by new game, load game, and tile test paths.

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};
use saltglass_steppe::trading::{calculate_area_tier, get_trade_interface};
use saltglass_steppe::ui::{UiState, handle_input};
use saltglass_steppe::{GameState, Renderer};
use std::io::{self, Stdout};

pub enum SessionOutcome {
    ReturnToMenu,
    Quit,
}

/// Run a game session loop. `on_turn` is called after each successful turn (use for IPC etc.).
pub fn run_game_session(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    renderer: &mut Renderer,
    mut state: GameState,
    mut on_turn: impl FnMut(&mut GameState, &UiState),
) -> io::Result<SessionOutcome> {
    let mut ui = UiState::new();
    ui.camera_x = state.player.x as f32;
    ui.camera_y = state.player.y as f32;
    ui.world_map_view.open = state.world.saved_on_world_map;

    // Initial tutorial check
    if let Some(msg) = state.get_next_tutorial_message() {
        ui.tutorial_message = Some(msg);
    }

    loop {
        if !ui.debug_console.active {
            ui.tick_frame();
            state.world.visual_effects.tick_hit_flash();
            state.world.visual_effects.tick_damage_numbers();
            state.world.visual_effects.tick_projectile_trails();
            state.world.visual_effects.tick_light_beams();
            state.world.visual_effects.tick_animation();
            ui.update_camera(state.player.x, state.player.y);
            ui.dialog_box.tick(16);
        }

        if let Some((speaker, text)) = state.pending_ui.dialogue.take() {
            ui.dialog_box.show(&speaker, &text);
        }
        if let Some((text, options)) = state.pending_ui.aria_dialogue.take() {
            ui.aria_interface.response_text = text;
            ui.aria_interface.options = options;
            ui.aria_interface.selected_option = 0;
        }
        if let Some(book_id) = state.pending_ui.book_open.take() {
            ui.book_reader.open(&book_id);
        }
        if let Some(trader_id) = state.pending_ui.trade.take() {
            if ui.dialog_box.active {
                state.pending_ui.trade = Some(trader_id);
            } else {
                let area_tier = calculate_area_tier(&state.world.enemies);
                if let Some(interface) = get_trade_interface(
                    &trader_id,
                    area_tier,
                    &state.player.faction_reputation,
                    None,
                ) {
                    ui.inventory_menu.close();
                    ui.quest_log.close();
                    ui.crafting_menu.close();
                    ui.wiki_menu.close();
                    ui.pause_menu.close();
                    ui.trade_menu.open(trader_id, interface);
                } else {
                    state.log("This merchant has nothing to trade.");
                }
            }
        }

        if let Some(ei) = ui.target_enemy
            && (ei >= state.world.enemies.len() || state.world.enemies[ei].hp <= 0)
        {
            ui.target_enemy = None;
        }

        if ui.show_controls {
            terminal.draw(super::render_controls)?;
            if event::poll(std::time::Duration::from_millis(16))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                ui.show_controls = false;
            }
        } else {
            terminal.draw(|frame| super::render(frame, &state, &mut ui, renderer))?;
            let action = handle_input(&mut ui, &mut state)?;
            match super::update(&mut state, action, &mut ui) {
                Some(true) => {
                    if ui.tutorial_message.is_none()
                        && let Some(msg) = state.get_next_tutorial_message()
                    {
                        ui.tutorial_message = Some(msg);
                    }
                    on_turn(&mut state, &ui);
                }
                Some(false) => return Ok(SessionOutcome::Quit),
                None => return Ok(SessionOutcome::ReturnToMenu),
            }
        }
    }
}
