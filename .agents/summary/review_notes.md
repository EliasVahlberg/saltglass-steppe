# Review Notes

<!-- Generated: 2026-04-06 | Consistency and completeness review -->

## Consistency Check

### Terminology Alignment

| Term | Used In | Status |
|------|---------|--------|
| "Verified State Store" | architecture.md, workflows.md, index.md | ✅ Consistent |
| "VERA" | Existing AGENTS.md, effects/mod.rs docstring | ⚠️ Legacy name — codebase has moved to "Verified State Store" but `effects/mod.rs` still says "VERA" in its module docstring, and the existing AGENTS.md uses VERA throughout. The consolidated AGENTS.md should use "Verified State Store" as the primary term with VERA as a parenthetical alias |
| "Bridge mutation" | architecture.md, data_models.md, interfaces.md | ✅ Consistent |
| "Atomic mutation" | architecture.md, data_models.md | ✅ Consistent |
| Command count "22" | interfaces.md, architecture.md | ✅ Matches `effects/mod.rs` Command enum |
| Mutation count "~70" | data_models.md, architecture.md | ✅ Matches `mutations.rs` |
| StateTransition count "8" | architecture.md | ✅ Matches `mutations.rs` |
| Turn phases "9" | architecture.md, workflows.md | ✅ Matches `TurnPhase::sequence()` |

### Cross-File Reference Integrity

| Reference | From | To | Status |
|-----------|------|----|--------|
| SYSTEM_STATUS.md | index.md, architecture.md | `docs/development/SYSTEM_STATUS.md` | ✅ File exists |
| VERIFIED_STATE_STORE.md | index.md | `docs/development/architecture_refactor/VERIFIED_STATE_STORE.md` | ✅ Referenced |
| ROADMAP.md | index.md | `docs/development/ROADMAP.md` | ✅ Referenced |
| DOCUMENT_DATABASE.md | index.md | `docs/DOCUMENT_DATABASE.md` | ✅ Referenced |

### Diagram Consistency

All documentation files use Mermaid diagrams exclusively. No ASCII art present. ✅

## Completeness Check

### Well-Covered Areas
- ✅ Core architecture (Verified State Store pattern, mutation model, cascade)
- ✅ Command dispatch and system handler interface
- ✅ Turn processing phases
- ✅ Combat flow with reaction cascade
- ✅ Tile generation pipeline
- ✅ Data file organization and cross-references
- ✅ DES testing interface
- ✅ Dependency usage and integration points

### Areas With Thin Coverage

| Area | Gap | Recommendation |
|------|-----|----------------|
| **Renderer internals** | components.md lists renderer modules but doesn't detail the rendering pipeline (tile → entity → lighting → particles → effects layering order) | Low priority — renderer is read-only and rarely modified |
| **UI input routing** | interfaces.md covers Command dispatch but not the UI-side input mapping (which keys map to which Commands in which contexts) | Medium priority — `ui/input.rs` has ~20 input contexts; a mapping table would help |
| **NPC dialogue system** | components.md mentions it briefly; no workflow diagram for dialogue tree traversal | Medium priority — dialogue is a complex subsystem with conditions, actions, ARIA interface |
| **Settlement generation details** | components.md covers it; workflows.md shows it in the pipeline but doesn't detail the internal settlement generation steps | Low priority — well-documented in `docs/features/SETTLEMENT_GENERATION.md` |
| **Multi-terminal IPC** | interfaces.md covers the protocol; no workflow diagram for satellite terminal lifecycle | Low priority — niche feature, well-contained in `ipc.rs` + `satellite.rs` |
| **Meta-progression** | Not mentioned in any summary file | Low priority — `meta.rs` handles cross-run unlocks, not core gameplay |
| **Dead code** | Not documented in summary files | Covered by SYSTEM_STATUS.md dead code section — no need to duplicate |

### Discrepancies With Existing AGENTS.md

| Item | Existing AGENTS.md | New Documentation | Resolution |
|------|--------------------|--------------------|------------|
| Architecture name | "VERA (Verified Effect-Rule Architecture)" | "Verified State Store" | Use "Verified State Store" — this is the current canonical name per `VERIFIED_STATE_STORE.md`. Mention VERA as historical alias |
| `effects/apply.rs` | Listed in directory map | File does not exist in current codebase — apply logic is in `state.rs::apply_one()` | Remove from directory map |
| Systems description | "Legacy systems (called via bridge effects)" | Systems are the primary handlers, called from dispatch.rs | Update description — systems are not legacy; they are the current pattern |
| Rule functions | Described as the primary VERA pattern | Being absorbed into `systems/` per SYSTEM_STATUS.md | Note that rules/ is legacy, systems/ is current |

## Recommendations

1. **Consolidated AGENTS.md** should use "Verified State Store" terminology, remove reference to non-existent `effects/apply.rs`, and correctly describe `systems/` as the primary handler layer (not legacy).

2. **Consolidated README.md** should be updated to reflect current architecture terminology and add the multi-terminal UI section which is a distinctive feature.

3. **Future documentation updates** should be triggered when:
   - New Command variants are added to `effects/mod.rs`
   - New Mutation variants are added to `mutations.rs`
   - New system handlers are added to `systems/`
   - New StateTransition variants are added or wired in `notify.rs`
