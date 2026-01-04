# Enhanced Main Questline Content

**Date:** 2026-01-02  
**Purpose:** Provide enhanced quest content that properly integrates the rich lore from narrative documents.

---

## Enhanced NPCs for Main Questline

### The Architect (Critical Missing NPC)

```json
{
  "id": "the_architect",
  "name": "The Architect", 
  "glyph": "Λ",
  "faction": "Heliograph Network",
  "description": "A holographic projection that shifts form based on the observer. To some, blueprints. To others, an angel of light.",
  "dialogue": [
    {
      "conditions": [{"adaptation_count_gte": 3}],
      "text": "Fascinating. The transformation proceeds ahead of schedule. Are you evolution or deviation?"
    },
    {
      "conditions": [{"faction_reputation": {"Mirror Monks": 75}}],
      "text": "The Monks see patterns where I see data. Perhaps that is why they survived the correction."
    },
    {
      "conditions": [{"has_item": "prime_lens_shard"}],
      "text": "You carry a fragment of the Prime Lens. The convergence approaches. Will you complete the work or shatter it forever?"
    },
    {
      "conditions": [],
      "text": "Input received. Analyzing trajectory. The probability of success remains... uncertain. Proceed?"
    }
  ],
  "actions": [
    {
      "id": "reveal_white_noon_truth",
      "name": "Query: Operation White Noon",
      "conditions": [{"has_item": "saint_key"}],
      "effect": {"story_flag": "white_noon_truth_revealed"},
      "response": "White Noon was not disaster. It was correction. The planet was dying. We saved it. The cost was... acceptable."
    },
    {
      "id": "offer_proxy_authority",
      "name": "Accept Proxy Authority Scan",
      "conditions": [{"story_flag": "white_noon_truth_revealed"}],
      "effect": {"story_flag": "proxy_authority_offered"},
      "response": "You may become a temporary saint-proxy. The authority is vast. The responsibility... absolute."
    }
  ]
}
```

### Forge-Master Kaine Durgan (Iron Covenant Leader)

```json
{
  "id": "forge_master_durgan",
  "name": "Forge-Master Kaine Durgan",
  "glyph": "K",
  "faction": "Iron Covenant", 
  "description": "Half-man, half-machine. Hydraulic treads replace his legs, a pneumatic hammer his right arm. Lead-lined armor bears scorch marks from countless battles against the light.",
  "dialogue": [
    {
      "conditions": [{"has_adaptation": "Prismhide"}],
      "text": "Another glitch. We can fix you. Strip away the glass. Make you human again. The procedure is... unpleasant. But necessary."
    },
    {
      "conditions": [{"faction_reputation": {"Glassborn": 50}}],
      "text": "You reek of their corruption. Glass-lover. Silicate sympathizer. You're part of the problem."
    },
    {
      "conditions": [{"adaptation_count_gte": 4}],
      "text": "Look at yourself. Is that human? The light has made you into something else. Something wrong."
    },
    {
      "conditions": [],
      "text": "Stop vibrating. Listen. The plan is simple. We cut the power. We kill the signal. We take our planet back."
    }
  ],
  "actions": [
    {
      "id": "offer_adaptation_removal",
      "name": "Remove Glass Corruption",
      "conditions": [{"has_adaptation": true}],
      "effect": {"remove_adaptation": "random"},
      "response": "The glass comes out screaming. But you'll thank me when you're human again."
    }
  ]
}
```

### The High Prism (Glass Prophets Leader)

```json
{
  "id": "the_high_prism",
  "name": "The High Prism",
  "glyph": "◊",
  "faction": "Glass Prophets",
  "description": "A being of pure light and geometry. Humanoid in shape but faceless, their body constantly refracts the environment. They float slightly above the ground.",
  "dialogue": [
    {
      "conditions": [{"adaptation_count_gte": 5}],
      "text": "You approach the threshold. Soon you will shed the flesh-cocoon entirely. We welcome you to the light."
    },
    {
      "conditions": [{"faction_reputation": {"Iron Covenant": 25}}],
      "text": "You still listen to the metal-bound. They fear what they cannot understand. Evolution is not optional."
    },
    {
      "conditions": [{"has_item": "shard_of_soul"}],
      "text": "You seek the Soul-Shard. It rests within us, as we rest within it. Prove your commitment to the light."
    },
    {
      "conditions": [],
      "text": "Do not fear the break. The break is how the light gets in. We offer you eternity. Why do you cling to the mud?"
    }
  ],
  "actions": [
    {
      "id": "accelerate_transformation",
      "name": "Embrace the Light",
      "conditions": [{"adaptation_count_gte": 3}],
      "effect": {"force_adaptation": true},
      "response": "Let the light flow through you. Become what you were always meant to be."
    }
  ]
}
```

---

## Enhanced Quest Content

### Act III: The Deep Archive Revelation

```json
{
  "id": "white_noon_revelation",
  "name": "The White Noon Revelation",
  "description": "Deep in the Archive, the truth awaits: White Noon was not disaster, but correction. The Heliograph still runs its incomplete loop, and the storms are its continuing attempt to finish what it started.",
  "category": "main",
  "act": 3,
  "objectives": [
    {
      "id": "access_deep_archive",
      "description": "Enter the Deep Archive Wing using your saint-key",
      "type": "reach",
      "location": "deep_archive_entrance"
    },
    {
      "id": "query_operation_white_noon",
      "description": "Access the classified Operation White Noon files",
      "type": "interact",
      "target": "archive_terminal_alpha"
    },
    {
      "id": "confront_the_architect",
      "description": "Interface with The Architect consciousness",
      "type": "talk_to",
      "npc_id": "the_architect"
    },
    {
      "id": "learn_heliograph_truth",
      "description": "Understand the true purpose of the Heliograph Network",
      "type": "story_revelation",
      "revelation_id": "heliograph_purpose"
    }
  ],
  "reward": {
    "xp": 500,
    "salt_scrip": 1000,
    "story_flags": ["white_noon_truth_revealed", "heliograph_purpose_known"],
    "unlocks_quests": ["the_prime_lens_prophecy", "vector_location_search"]
  },
  "criteria": {
    "requires_quests_completed": ["custodians_query"],
    "required_items": ["saint_key"],
    "min_faction_reputation": {"Archive": 0}
  }
}
```

### The Prime Lens Prophecy Quest

```json
{
  "id": "the_prime_lens_prophecy",
  "name": "The Prime Lens Prophecy", 
  "description": "The Architect speaks of a master key—the Prime Lens—shattered into three aspects during White Noon. To control the Heliograph, you must gather the Shards of Clarity, Will, and Soul.",
  "category": "main",
  "act": 3,
  "objectives": [
    {
      "id": "discover_prophecy",
      "description": "Learn about the Prime Lens prophecy from Archive records",
      "type": "story_revelation",
      "revelation_id": "prime_lens_prophecy"
    },
    {
      "id": "locate_shard_of_clarity",
      "description": "Find the Shard of Clarity in the Archive Core databanks",
      "type": "collect",
      "item_id": "shard_of_clarity",
      "count": 1
    },
    {
      "id": "locate_shard_of_will", 
      "description": "Retrieve the Shard of Will from the Magma-Glass Caverns",
      "type": "collect",
      "item_id": "shard_of_will",
      "count": 1
    },
    {
      "id": "locate_shard_of_soul",
      "description": "Obtain the Shard of Soul from The High Prism",
      "type": "collect", 
      "item_id": "shard_of_soul",
      "count": 1
    }
  ],
  "reward": {
    "xp": 1000,
    "salt_scrip": 2000,
    "story_flags": ["prime_lens_aspects_gathered"],
    "unlocks_quests": ["the_vector_choice"]
  },
  "criteria": {
    "requires_quests_completed": ["white_noon_revelation"],
    "story_flags": ["white_noon_truth_revealed"]
  }
}
```

---

## Prime Lens Aspect Items

### Shard of Clarity (The Mind)

```json
{
  "id": "shard_of_clarity",
  "name": "Shard of Clarity",
  "glyph": "◇",
  "description": "Pure crystallized information. It hums with the weight of understanding, revealing the true code beneath reality's surface.",
  "value": 10000,
  "weight": 0,
  "usable": false,
  "prime_lens_aspect": "mind",
  "special_properties": {
    "reveals_archive_secrets": true,
    "grants_code_comprehension": true,
    "required_for_ending": ["seal_heliograph", "recalibrate_system"]
  }
}
```

### Shard of Will (The Body)

```json
{
  "id": "shard_of_will", 
  "name": "Shard of Will",
  "glyph": "◆",
  "description": "Pure crystallized energy. It pulses with barely contained power, the strength to endure what would destroy lesser beings.",
  "value": 10000,
  "weight": 0,
  "usable": false,
  "prime_lens_aspect": "body",
  "special_properties": {
    "grants_energy_resistance": true,
    "enables_heliograph_interface": true,
    "required_for_ending": ["claim_sainthood", "amplify_transformation"]
  }
}
```

### Shard of Soul (The Spirit)

```json
{
  "id": "shard_of_soul",
  "name": "Shard of Soul", 
  "glyph": "◈",
  "description": "Pure crystallized resonance. It sings with the harmony of transformation, the will to command change itself.",
  "value": 10000,
  "weight": 0,
  "usable": false,
  "prime_lens_aspect": "spirit",
  "special_properties": {
    "grants_transformation_control": true,
    "enables_storm_command": true,
    "required_for_ending": ["recalibrate_system", "amplify_transformation"]
  }
}
```

---

## Enhanced Faction Integration

### Iron Covenant Questline

```json
{
  "id": "iron_covenant_introduction",
  "name": "The Iron Path",
  "description": "Forge-Master Durgan offers a different solution: strip away the glass corruption entirely. Return humanity to its pure, unmodified state.",
  "category": "faction",
  "act": 2,
  "objectives": [
    {
      "id": "meet_forge_master",
      "description": "Meet Forge-Master Durgan in the Magma-Glass Caverns",
      "type": "talk_to",
      "npc_id": "forge_master_durgan"
    },
    {
      "id": "witness_purification",
      "description": "Witness the 'purification' of a volunteer Glassborn",
      "type": "cutscene",
      "cutscene_id": "adaptation_removal_procedure"
    },
    {
      "id": "choose_iron_path",
      "description": "Decide whether to support the Iron Covenant's methods",
      "type": "choice",
      "choice_id": "support_iron_covenant"
    }
  ],
  "reward": {
    "faction_reputation": {"Iron Covenant": 50},
    "items": ["anti_glass_equipment"],
    "unlocks_quests": ["purification_campaign"]
  },
  "criteria": {
    "requires_quests_completed": ["the_broken_key"],
    "max_adaptation_count": 3
  }
}
```

---

This enhanced content provides the foundation for a main questline that properly integrates the rich lore established in the narrative documents while maintaining the solid mechanical structure of the original quest spine.
