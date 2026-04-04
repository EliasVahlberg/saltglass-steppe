# dependencies.md

## Runtime Dependencies

### UI & Terminal

| Crate | Version | Usage |
|-------|---------|-------|
| ratatui | 0.28 | TUI framework — multi-panel layouts, widgets, rendering |
| crossterm | 0.28 | Cross-platform terminal backend — input, colors, cursor |

### Procedural Generation

| Crate | Version | Usage |
|-------|---------|-------|
| terrain-forge | 0.7.0 | 2D procedural terrain generation (base terrain layer) |
| bracket-noise | 0.8.7 | Perlin/simplex noise for terrain variation |
| bracket-pathfinding | 0.8 | A* pathfinding for connectivity and road generation |
| bracket-geometry | 0.8.7 | Geometric utilities (points, lines, distances) |
| bracket-algorithm-traits | 0.8.7 | Trait definitions for bracket-lib integration |

### Serialization & Data

| Crate | Version | Features | Usage |
|-------|---------|----------|-------|
| serde | 1.0 | `derive` | Serialization framework |
| serde_json | 1.0 | — | JSON for configs, saves, DES scenarios |
| ron | 0.8 | — | Rusty Object Notation for meta_progress |
| jsonschema | 0.17 | — | Runtime JSON schema validation |
| schemars | 0.8 | `derive` | JSON schema generation from Rust types |

### RNG

| Crate | Version | Usage |
|-------|---------|-------|
| rand | 0.8 | Random number generation traits and distributions |
| rand_chacha | 0.3 | ChaCha8Rng — seeded deterministic RNG for all game systems |

### Utilities

| Crate | Version | Features | Usage |
|-------|---------|----------|-------|
| clap | 4.0 | `derive` | CLI argument parsing |
| once_cell | 1.0 | — | Lazy static initialization for data caches |
| rayon | 1.11 | — | Data parallelism for generation |
| textwrap | 0.16 | — | Text wrapping for UI panels |
| chrono | 0.4 | `serde` | Timestamps for save files |
| which | 6.0 | — | Terminal emulator detection for satellite spawning |
| image | 0.25 | — | Map image export |
| smallvec | 1.15 | — | Stack-allocated vectors for hot paths |
| md5 | 0.8 | — | Save file checksum computation |

## Dev Dependencies

| Crate | Version | Usage |
|-------|---------|-------|
| tempfile | 3.24 | Temporary files/directories for save/load tests |

## Build Configuration

```toml
[profile.release]
codegen-units = 1    # Single codegen unit for better optimization
lto = true           # Link-time optimization
opt-level = 3        # Maximum speed (TUI rendering performance)
strip = true         # Remove debug symbols from binary
panic = "abort"      # No unwinding — reduces binary size
```

## External Tools

| Tool | Required For | Install |
|------|-------------|---------|
| rustfmt | Code formatting (CI enforced) | `rustup component add rustfmt` |
| clippy | Linting with `-D warnings` (CI enforced) | `rustup component add clippy` |
| jq | JSON data file validation scripts | System package manager |
| mingw-w64 | Cross-compile Linux → Windows | `apt install mingw-w64` + `rustup target add x86_64-pc-windows-gnu` |
| Zig | Multi-target release builds (`build-release.sh`) | [ziglang.org](https://ziglang.org) |
