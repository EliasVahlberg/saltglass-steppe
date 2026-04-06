# Side Note: Renderer Threading

> Status: DESIGN NOTE — not part of ESCAEV migration scope
> Date: 2026-04-04
> Relates to: Architecture refactor, but independent of it

## Principle

The renderer is read-only. It reads game state and produces frames. It never mutates game state. This boundary must be preserved after any refactoring.

Because rendering is inherently parallel work (composing tiles, lighting, particles, effects) and the game loop is turn-based, the renderer should run on a separate thread to avoid stuttering during complex scenes (storms, particle effects, large maps on the ultrawide).

## Current Architecture

```
Main thread (synchronous):
  loop {
    input = read_input()          // blocks waiting for keypress
    update_state(input)           // game logic
    renderer.render(&state)       // render frame — blocks until done
  }
```

Everything is single-threaded. Rendering blocks the game loop. If a frame takes long (complex lighting, many particles), input feels sluggish.

## Proposed Architecture

```
Game thread:
  loop {
    input = read_input()
    update_state(input)
    render_channel.send(state.snapshot())  // non-blocking
  }

Render thread (owns terminal backend):
  loop {
    snapshot = render_channel.recv()       // blocks until new state
    renderer.render(&snapshot)             // draws to terminal
  }
```

The game thread never waits for rendering. The render thread always draws the most recent state. If the game thread produces states faster than the render thread can draw (unlikely in turn-based), the render thread skips intermediate states and draws the latest.

## Implementation Notes

### State snapshot

The render thread needs a consistent read of game state. Options:

1. **Clone the render-relevant subset.** Don't clone all of GameState — clone only what the renderer reads: map tiles, entity positions, lighting, particles, animations, player position, HUD data. Define a `RenderSnapshot` struct with just these fields.

2. **`Arc<RwLock<RenderSnapshot>>`** — game thread writes, render thread reads. Simple, minimal allocation. The lock is held briefly (snapshot write is fast).

3. **Channel with `RenderSnapshot`** — game thread sends snapshots, render thread receives. No lock contention. Slightly more allocation but cleaner separation.

Option 3 (channel) is the cleanest for this codebase. Use `std::sync::mpsc` or `crossbeam::channel`.

### Terminal backend ownership

ratatui/crossterm requires that terminal I/O happens on one thread. The render thread must own the `Terminal<CrosstermBackend>` exclusively. The game thread communicates input via crossterm's event polling (which can happen on either thread, but should be on the game thread since it drives the game loop).

### Frame rate

The render thread can run at a target frame rate (e.g., 60fps) independent of game ticks. Between player inputs, it re-renders the same state (for animations, particles, weather effects). This makes ambient effects smooth without requiring the game loop to tick.

```
Render thread:
  loop {
    if let Ok(new_snapshot) = channel.try_recv() {
      current = new_snapshot;
    }
    renderer.render(&current);
    sleep_until_next_frame(target_fps: 60);
  }
```

### What this means for ESCAEV

Nothing. ESCAEV operates on the game thread: commands, rules, effects, application — all synchronous, single-threaded. The render thread only reads the post-application state. The Trace is game-thread-only. PresentationEffects (hit flash, damage numbers, particles) are applied to state on the game thread; the render thread picks them up in the next snapshot.

The renderer's read-only contract is the boundary. ESCAEV doesn't cross it. Threading doesn't cross it. They're independent concerns.

## When to do this

**After the architecture refactor, not during.** Reasons:

1. Threading adds complexity. Doing it simultaneously with the ESCAEV migration creates two moving targets.
2. The current synchronous renderer works. It's not a blocking issue — it's a polish issue.
3. The `RenderSnapshot` struct design benefits from knowing the final shape of GameState after sub-state extraction (Phase 3.5). If you thread the renderer before sub-states are extracted, you'll clone more than necessary.

Suggested timing: after ESCAEV Phase 4 (end_turn decomposition), before content expansion. The refactored state makes the snapshot boundary cleaner.
