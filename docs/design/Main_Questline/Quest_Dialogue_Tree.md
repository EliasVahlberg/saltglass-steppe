# Quest Dialogue Trees

**Purpose:** To map the branching narrative paths for the critical moments in the Main Quest.

---

## Act 1 Climax: The Nexus Meeting

**Context:** The Player has just repaired the Signal Tower and arrived at the Nexus Plateau. They meet the three faction leaders for the first time.

**NPCs:** Durgan (Iron), Prism (Glass), Matthias (Synthesis).

**Start Node:** `nexus_intro`

- **Matthias:** "Welcome, traveler. You have opened the way. The signal is clear."
- **Durgan:** "Clear? It's noisy. Just like you, monk. Traveler, did you find any salvage?"
- **Prism:** "Silence, iron-man. The traveler shines. I see the potential for refraction."

**Player Options:**

1.  **[To Durgan]** "I found old tech. It needs fixing." -> **Path: Iron Sympathy**
    - _Durgan:_ "Good. You have eyes. Come to the Bunker. We'll put that scrap to use."
    - _Effect:_ +10 Iron Rep, Unlock "Magma Descent" quest first.
2.  **[To Prism]** "The light guided me here." -> **Path: Glass Sympathy**
    - _Prism:_ "The light guides all. Come to the Gardens. We will polish your soul."
    - _Effect:_ +10 Glass Rep, Unlock "Glass Pilgrimage" quest first.
3.  **[To Matthias]** "I just want to survive." -> **Path: Neutral**
    - _Matthias:_ "Survival is the first step to wisdom. Stay here. Rest. Learn."
    - _Effect:_ +5 All Rep, Unlock "Archive Heist" quest first.

---

## Act 2: The Archive Heist (The Shard of Clarity)

**Context:** The Player reaches the Logic-Gatekeeper AI guarding the Shard.

**NPC:** Logic-Gatekeeper (AI).

**Start Node:** `gatekeeper_confrontation`

- **Gatekeeper:** "UNAUTHORIZED ACCESS. BIOLOGICAL ENTITY DETECTED. STATE PURPOSE."

**Player Options:**

1.  **[Hack]** "Admin Override: Code Helios-7." (Requires Tech > 20)
    - _Success:_ "OVERRIDE ACCEPTED. RETRIEVING SHARD." -> **Get Shard (Peaceful)**
    - _Fail:_ "INVALID CODE. DEPLOYING DRONES." -> **Combat Encounter**
2.  **[Combat]** "My purpose is to dismantle you."
    - _Effect:_ **Combat Encounter** -> **Get Shard (Violent)**
3.  **[Lore]** "I seek the memory of the Architect." (Requires 'The Echoes of Sector 7' item)
    - _Gatekeeper:_ "ARCHITECT SIGNATURE RECOGNIZED. PROCEED." -> **Get Shard (Lore)**

---

## Act 3 Climax: The Choice of the Lens

**Context:** The Player stands at the top of the Spire. The Heliograph Control Panel is open. The Architect is watching.

**NPC:** The Architect.

**Start Node:** `final_choice`

- **Architect:** "The Prime Lens is seated. The array is awaiting calibration. What is your directive?"

**Player Options:**

1.  **[Insert Iron Chip]** "Initiate Null-Pulse. End the storms."
    - _Architect:_ "Warning. This will purge all quantum data. Irreversible. Confirm?"
    - _Choice:_ **CONFIRM** -> **Ending A (Restoration)**
2.  **[Insert Prism Shard]** "Initiate Full-Burn. Ascend."
    - _Architect:_ "Acknowledged. Maximizing output. Prepare for sublimation."
    - _Choice:_ **CONFIRM** -> **Ending B (Ascension)**
3.  **[Tune Frequency]** "Calibrate for Synthesis. 50% Output." (Requires Clarity + Will + Soul Shards)
    - _Architect:_ "Calculating... Solution found. Implementing Harmony Protocol."
    - _Choice:_ **CONFIRM** -> **Ending C (Synthesis)**
4.  **[Smash Lens]** "No one should have this power."
    - _Architect:_ "Critical Failure. System destabilizing. Goodbye."
    - _Choice:_ **DO IT** -> **Ending D (Void)**
