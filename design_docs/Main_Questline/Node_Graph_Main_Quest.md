# Node Graph --- Saltglass Steppe Main Quest Spine

_Acts as nodes + dependencies + fail-forward alternatives (roguelike-safe)._

## Legend

- Node ID: `A1.N3` = Act 1, Node 3
- Type: `Story`, `Gate`, `Location`, `Boss/Setpiece`, `Faction`
- Unlock: what the node grants (item/flag/location)
- Hard Req: must have to enter/complete in the cleanest way
- Alt (Fail-forward): other ways to progress if you miss/lose/anger a faction
- Costs/Consequences: reputation/refraction/drone escalation, etc.

---

## Global State Flags (Used by Many Nodes)

- `KEY_BROKEN` --- damaged saint-key obtained
- `KEY_INTACT` --- intact saint-key obtained
- `AUTH_POINTS` --- count of "authority proofs" (0--3) used to open Deep Wing
- `TEAR_HELD` / `TEAR_CONSUMED` --- Saint's Tear state
- `FORECAST_TOOL` --- storm forecast instrument built
- `ANGLE_LENS` --- Angle-Split Lens obtained
- `SEAM_PASSAGE` --- Glassborn seam-route access
- `DRONE_ALERT` --- escalating security (0--3+) based on Archive friction + adaptation level
- `FACTION_LEAN` --- Monk/Engineer/Glassborn bias (soft until Act III)

---

# ACT I --- Salt Testament (White Noon's Echo)

### `A1.N1` --- Spawn: The Counting Sky

- Type: Story (tutorial framing)
- Unlock: Storm timer awareness; first "storm rewrite" preview
- Deps: None
- Fail-forward: None (cannot fail)

---

### `A1.N2` --- The Pilgrim's Last Angle

- Type: Story / Location
- Goal: Find dying pilgrim; obtain shard + "door remembers" rumor
- Unlock: `PILGRIM_SHARD` (Scripture Shard `?`), pointer to reliquary
- Deps: `A1.N1`
- Alt (Fail-forward):
  - If pilgrim dies before you reach them: find their body post-storm as a Salt Mummy `m` miniboss holding the shard.
  - If storm overwrites the route: shard relocates to nearest "fused stack" room (still reachable).
- Costs/Consequences: None (but storm pressure teaches urgency)

---

### `A1.N3` --- Saint's Eye Reliquary

- Type: Location / Setpiece
- Goal: Retrieve Saint's Tear `○` (and optional relics)
- Unlock: `TEAR_HELD` and/or `TEAR_CONSUMED`
- Deps: `A1.N2`
- Alt (Fail-forward):
  - If you cannot survive glare tiles: buy/loot Veil Tincture `!` first (optional side node `A1.S1`).
  - If you consume the Tear early: gain safer drone interactions but lose barter leverage later (still winnable).
- Costs/Consequences: Consuming Tear reduces Refraction (slows adaptation gates).

---

### `A1.N4` --- The First Key (Broken)

- Type: Gate / Item
- Goal: Acquire damaged saint-key `⚷`
- Unlock: `KEY_BROKEN`, `AUTH_POINTS +1` (counts as partial)
- Deps: `A1.N3` (soft) --- can be found without Tear, but reliquary rumor points here.
- Alt (Fail-forward):
  - If key is missed: can be looted from a destroyed drone later (`A2.S2 Drone Husk`) at higher `DRONE_ALERT`.
- Costs/Consequences: Sets `DRONE_ALERT = 1` baseline (you are "incomplete credential").

---

### `A1.N5` --- Thesis: White Noon is Named

- Type: Story Flag
- Goal: First Archive fragment mentioning *OPERATION: WHITE NOON*
- Unlock: `WHITE_NOON_NAMED`
- Deps: `A1.N4` (or any Archive interaction)
- Alt (Fail-forward):
  - Without Archive access: Monk scripture interpretation can reveal the phrase "White Noon" as a liturgical title (less precise but progresses the spine).
- Costs/Consequences: None

---

# ACT II --- Keys of the Unfinished Saints (Authority Building)

Act II Objective: reach `AUTH_POINTS >= 2` and obtain one of: `ANGLE_LENS` or `FORECAST_TOOL` or `SEAM_PASSAGE` (route capability), then secure `KEY_INTACT` _or_ an equivalent override.

---

## Act II Hub: Choose Your Authority Path(s)

These three nodes can be done in any order; you only need enough to progress, but doing more changes options later.

### `A2.N1` --- Monk Path: The Choir of Shards

- Type: Faction / Location
- Goal: Collect 3 specific Scripture Shards `?` ("credential prayer" angles)
- Unlock: `AUTH_POINTS +1` (liturgical authority) + Monk reputation
- Hard Req: None
- Alt (Fail-forward):
  - If you anger Monks: steal shards from their storm-chapel during a storm (harder combat + reputation penalty).
- Costs/Consequences: High adaptation increases Monk reverence → may trigger obligation quests.

---

### `A2.N2` --- Engineer Path: Crucible Block Salvage

- Type: Faction / Location
- Goal: Recover optics + couplers; build Storm Forecast Instrument
- Unlock: `FORECAST_TOOL`, `AUTH_POINTS +1` (hardware authority) + Engineer reputation
- Hard Req: Access to Crucible Block (can be storm-revealed)
- Alt (Fail-forward):
  - If you cannot build tool: acquire a prebuilt forecast core from a black-market hermit cache (`A2.S1`).
- Costs/Consequences: Forecast tool makes storms more "readable"; Engineers may demand it be shared.

---

### `A2.N3` --- Glassborn Path: Seam-Kinship

- Type: Faction / Rite / Setpiece
- Goal: Survive a high-intensity storm in a marked zone ("Crucible-adjacent rite")
- Unlock: `SEAM_PASSAGE`, `AUTH_POINTS +1` (kinship authority) + guaranteed adaptation pick
- Hard Req: Reach a marked zone before storm hits
- Alt (Fail-forward):
  - If you refuse the rite: earn `SEAM_PASSAGE` by paying Storm Glass `◆` to Sable-of-the-Seam (hard economy gate).
- Costs/Consequences: Increases refraction/adaptation → raises `DRONE_ALERT` growth rate.

---

## Key Acquisition Cluster (choose one "main" route, others remain optional)

### `A2.N4` --- The Second Key (Intact)

- Type: Gate / Item
- Goal: Obtain intact saint-key `⚷`
- Unlock: `KEY_INTACT`
- Deps: Any two of (`A2.N1`, `A2.N2`, `A2.N3`) *or* `AUTH_POINTS >= 2`
- Alt (Fail-forward) routes:
  1.  Monk route: Halix grants it for completing the storm-walk interpretation.
  2.  Engineer route: extract from sealed drone hub using forecast timing.
  3.  Glassborn route: trade with Glassborn Merchant; price scales with adaptation.
  4.  Rogue route: kill a high-tier Archive Drone patrol leader (hard combat; spikes `DRONE_ALERT`).
- Costs/Consequences: Possessing intact key reduces some drone hostility but increases attention (you are "impossible credential").

---

### `A2.N5` --- Approach IRI-7

- Type: Story / Gate (social)
- Goal: Contact Archive Custodian IRI-7 for Deep Wing access conditions
- Unlock: `CUSTODIAN_CONTACT`
- Deps: `A1.N5` and (`KEY_BROKEN` or `KEY_INTACT` or `AUTH_POINTS >= 2`)
- Alt (Fail-forward):
  - If you cannot reach IRI-7 due to drones: use Veil Tincture to enter as "unmarked" (temporary stealth access).
- Costs/Consequences: IRI-7 logs your biology; high adaptation may pre-set `DRONE_ALERT +1`.

---

# ACT III --- The Custodian's Query (Heliograph Truth)

### `A3.N1` --- Deep Wing Entry: OPERATION WHITE NOON

- Type: Gate / Dungeon
- Goal: Enter Deep Archive Wing
- Hard Req (clean): `KEY_INTACT` + `AUTH_POINTS >= 2`
- Alt (Fail-forward):
  - `KEY_BROKEN` + `AUTH_POINTS >= 3` (patchwork authority)
  - No key: brute-force breach with Glass Pick `/` + Engineer toolchain (sets `DRONE_ALERT = 3`, heavy combat)
  - Monk override: liturgical "temporary pass" + Veil Tincture (time-limited entry)
- Unlock: `DEEP_WING_ACCESS`
- Costs/Consequences: Deep Wing continuously escalates drones unless you move decisively.

---

### `A3.N2` --- Revelation Node: White Noon Directive Chain

- Type: Story / Terminal query
- Goal: Reconstruct the directive chain (what / why / still running)
- Unlock: `WHITE_NOON_TRUTH` (3-part revelation)
- Deps: `A3.N1`
- Alt (Fail-forward):
  - If terminal is destroyed or inaccessible: collect 3 "mirror-memory" data fragments from different Deep Wing rooms; IRI-7 can assemble them later (slower but safe).
- Costs/Consequences: None, aside from dungeon risk.

---

### `A3.N3` --- IRI-7 Offer: Proxy Authority Scan

- Type: Choice / Permanent consequence
- Goal: Accept or refuse becoming temporary "saint proxy"
- Unlock (if accept): `PROXY_AUTH` (new commands vs Archive systems), but higher social marking
- Deps: `A3.N2`
- Alt (Fail-forward):
  - If you refuse: you can still proceed to endgame, but with fewer control options at the Vector.
- Costs/Consequences: Accepting may permanently raise `DRONE_ALERT` floor and intensify Monk/Engineer pressures.

---

### `A3.N4` --- Obtain the Heliograph Vector

- Type: Story / Key Item
- Goal: Acquire Vector coordinates + timing window
- Unlock: `VECTOR_LOCATED`
- Deps: `A3.N2` (and sometimes `A3.N3` depending on run)
- Alt (Fail-forward):
  - If you fail to extract full coordinates: obtain partial vector + use one faction's expertise to complete it:
    - Monks complete via prophecy angles,
    - Engineers via forecast math,
    - Glassborn via seam-route memory.
- Costs/Consequences: Sets endgame clock: you must reach Vector before next major Patterning cycle.

---

# ACT IV --- The Vector Choice (Endgame)

### `A4.N1` --- Vector Spire Approach

- Type: Endgame hub / Route selection
- Goal: Choose approach route that defines hazards/enemy mix
- Deps: `VECTOR_LOCATED`
- Route Unlocks:
  - Monk route easier if Monk rep high or `ANGLE_LENS`
  - Engineer route easier if `FORECAST_TOOL`
  - Glassborn route easier if `SEAM_PASSAGE`
- Fail-forward: Any route can be brute-forced; storm edits may open emergency seams.

---

### `A4.N2` --- Vector Arena: The Storm Window

- Type: Boss/Setpiece (storm + drones + wraiths)
- Goal: Survive long enough to execute one intervention
- Deps: `A4.N1`
- Alt (Fail-forward):
  - If you arrive late: you can "ride" the storm by taking heavy Refraction risk---harder fight, still possible.
- Costs/Consequences: Typically pushes you over an adaptation threshold right before the final choice.

---

## Final Choice Node (Irreversible)

### `A4.N3` --- Intervention Choice

- Type: Endgame choice (4 endings)
- Deps: `A4.N2`

#### `A4.E1` --- Seal the Heliograph (End Storms)

- Req (clean): `KEY_INTACT` + IRI-7 protocol compliance (or `PROXY_AUTH`)
- Alt: Sacrifice Saint's Tear + Storm Glass as "shutdown payment" (rare resources)

#### `A4.E2` --- Recalibrate (Predictable Storms)

- Req (clean): `FORECAST_TOOL` or Engineer rep high
- Alt: Monk lens-ritual using `ANGLE_LENS` + 3 Scripture Shards

#### `A4.E3` --- Claim Sainthood (Become Living Credential)

- Req (clean): `PROXY_AUTH` accepted at `A3.N3`
- Alt: Perform a dangerous "self-scan" without IRI-7 (massive `DRONE_ALERT`, heavy refraction cost)

#### `A4.E4` --- Break the Vector (Amplify the Loop)

- Req (clean): High Refraction / multiple adaptations (world state gate)
- Alt: Glassborn rite at the Vector with Sable-of-the-Seam present (requires Glassborn rep)

---

# Fail-Forward Summary (So the Spine Doesn't Collapse)

## If you never get an intact saint-key

You can still reach the Deep Wing via:

- `AUTH_POINTS >= 3` + broken key patchwork, or
- Engineer breach path (combat + `DRONE_ALERT` spike), or
- Monk temporary pass (time-limited, higher tension).

## If you anger/lose a faction

- Another faction can supply the missing "authority proof."
- Rogue routes (steal, salvage, kill-drone) exist but raise danger and tighten resources.

## If storms erase routes repeatedly

- Each critical objective has at least one relocation rule:
  - Key items reappear in nearest "protected" tileset (Archive vault, fused library, sealed reliquary).
  - NPC anchors can shift between known shelters after storms (they "migrate" like the player must).

---

# Optional: Minimal Node List (for Implementation Tracking)

- Act I: `A1.N1 → A1.N2 → A1.N3 → A1.N4 → A1.N5`
- Act II: `A2.N1 | A2.N2 | A2.N3` (any order) → `A2.N4` → `A2.N5`
- Act III: `A3.N1 → A3.N2 → (A3.N3 optional) → A3.N4`
- Act IV: `A4.N1 → A4.N2 → A4.N3 (E1/E2/E3/E4)`
