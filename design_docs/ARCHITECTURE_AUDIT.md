# Architecture Audit: Coupling Analysis

## Executive Summary

**Status:** Critical issues resolved ✅
**Remaining:** Moderate issues (can be addressed incrementally)

---

## Resolved Issues

### ✅ 1. Visual Effects Mixed with Game State (FIXED)

**Change:** `TriggeredEffect.frames_remaining` → `turns_remaining`
- Effects now tick in `GameState::tick_turn()` instead of render loop
- Effect durations changed from frames (20, 30) to turns (2, 3)
- DES can now properly simulate visual effects

### ✅ 2. UI State Mixed with Game State (FIXED)

**Change:** Created `UiState` struct in main.rs
- Contains: `look_mode`, `frame_count`, `show_controls`
- Separate from `GameState` (game logic only)
- Functions updated: `handle_input()`, `update()`, `render()`

---

## Remaining Moderate Issues (Future Work)

### 3. No Input Abstraction Layer

**Location:** `src/main.rs:40-80`

**Problem:** Input handling directly coupled to crossterm events.

**Impact:** Low - DES bypasses input system by design.

**Future Fix:** Create `InputSource` trait for alternative input sources.

---

### 4. Renderer Tightly Coupled to Terminal

**Location:** `src/main.rs:100-250`

**Problem:** `render()` uses ratatui types directly.

**Impact:** Low - DES doesn't need rendering.

**Future Fix:** Create `Renderer` trait for different backends.

---

### 5. Save/Load in Update Function

**Location:** `src/main.rs:70-85`

**Impact:** Low - Works correctly, just not ideal separation.

**Future Fix:** Extract to persistence layer.

---

## Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Input     │  │  Renderer   │  │     Game Loop       │  │
│  │ (crossterm) │  │  (ratatui)  │  │                     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│         ▼                ▼                     ▼             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  ┌───────────┐  ┌───────────┐                           ││
│  │  │ GameState │  │  UiState  │  ← DECOUPLED              ││
│  │  │ (turns,   │  │ (look,    │                           ││
│  │  │  combat,  │  │  frame,   │                           ││
│  │  │  effects) │  │  menu)    │                           ││
│  │  └───────────┘  └───────────┘                           ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘

DES operates directly on GameState, bypassing UI entirely:
┌─────────────────────────────────────────────────────────────┐
│                        DES                                   │
│  ┌─────────────┐                  ┌─────────────────────┐   │
│  │ Scenario    │ ───────────────► │   GameState         │   │
│  │ (JSON)      │                  │   (same as game)    │   │
│  └─────────────┘                  └─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Test Coverage

- 20 unit tests (lib.rs)
- 5 DES unit tests (des/mod.rs)  
- 3 integration tests (tests/des_scenarios.rs)
- All 23 tests pass after refactoring

