# 4 Storm + Map-Edit Systems Specification (Technical Design Doc)

---

Audience: Engineering, systems design, QA Owner: Systems Designer + Lead Engineer Purpose: Turn the signature feature into implementable, testable behavior.

Contents:

- Storm "forecast" inputs and how players receive warnings
- Allowed edit operations (rotate module, swap modules, fuse walls, deposit nodes, spawn entities)
- Constraints (max change radius, disallowed edits, preservation of critical paths)
- Determinism rules (seed behavior; save/load implications)
- Post-storm "diff report" UX requirements
- Edge cases (quest rooms, stairs, locked doors, NPCs mid-room)

Outputs: Implementable spec + acceptance tests for QA.

---
