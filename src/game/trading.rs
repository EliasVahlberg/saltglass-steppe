use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

/// Trading system with faction, tier, and relationship-based availability
#[derive(Clone, Debug, Deserialize)]
pub struct TraderTable {
    pub trader_id: String,
    pub name: String,
    pub faction: String,
    pub base_tier: u32,
    pub items: Vec<TradeItem>,
    #[serde(default)]
    pub reputation_modifiers: HashMap<String, ReputationModifier>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TradeItem {
    pub item_id: String,
    pub base_price: u32,
    pub stock: i32, // -1 = infinite
    #[serde(default)]
    pub min_tier: u32,
    #[serde(default)]
    pub max_tier: Option<u32>,
    #[serde(default)]
    pub required_reputation: i32,
    #[serde(default)]
    pub faction_exclusive: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReputationModifier {
    pub price_multiplier: f32,
    pub stock_bonus: i32,
    #[serde(default)]
    pub exclusive_items: Vec<String>,
}

#[derive(Deserialize)]
struct TradersFile {
    traders: Vec<TraderTable>,
}

static TRADERS: Lazy<HashMap<String, TraderTable>> = Lazy::new(|| {
    let data = include_str!("../../data/traders.json");
    let file: TradersFile = serde_json::from_str(data).expect("Failed to parse traders.json");
    file.traders.into_iter().map(|t| (t.trader_id.clone(), t)).collect()
});

/// Available trade item with calculated price and stock
#[derive(Clone, Debug)]
pub struct AvailableTradeItem {
    pub item_id: String,
    pub price: u32,
    pub stock: i32,
    pub base_price: u32,
}

/// Trading interface for a specific trader
#[derive(Clone, Debug)]
pub struct TradeInterface {
    pub trader_id: String,
    pub trader_name: String,
    pub available_items: Vec<AvailableTradeItem>,
    pub can_sell_to: bool,
    pub sell_price_multiplier: f32,
}

pub fn get_trader(trader_id: &str) -> Option<&'static TraderTable> {
    TRADERS.get(trader_id)
}

pub fn all_trader_ids() -> Vec<&'static str> {
    TRADERS.keys().map(|s| s.as_str()).collect()
}

/// Calculate current area tier based on enemy difficulty
pub fn calculate_area_tier(enemies: &[crate::game::Enemy]) -> u32 {
    if enemies.is_empty() {
        return 1;
    }
    
    let avg_hp: f32 = enemies.iter().map(|e| e.hp as f32).sum::<f32>() / enemies.len() as f32;
    
    match avg_hp as u32 {
        0..=20 => 1,
        21..=40 => 2,
        41..=60 => 3,
        61..=80 => 4,
        _ => 5,
    }
}

/// Get trading interface for a trader based on current game state
pub fn get_trade_interface(
    trader_id: &str,
    area_tier: u32,
    faction_reputation: &HashMap<String, i32>,
    player_faction: Option<&str>,
) -> Option<TradeInterface> {
    let trader = get_trader(trader_id)?;
    let player_rep = faction_reputation.get(&trader.faction).unwrap_or(&0);
    
    // Calculate reputation modifier
    let default_modifier = ReputationModifier {
        price_multiplier: 1.0,
        stock_bonus: 0,
        exclusive_items: Vec::new(),
    };
    
    let rep_modifier = trader.reputation_modifiers.iter()
        .find(|(rep_str, _)| {
            let threshold: i32 = rep_str.parse().unwrap_or(0);
            *player_rep >= threshold
        })
        .map(|(_, modifier)| modifier)
        .unwrap_or(&default_modifier);
    
    let mut available_items = Vec::new();
    
    for item in &trader.items {
        // Check tier requirements
        if item.min_tier > area_tier {
            continue;
        }
        if let Some(max_tier) = item.max_tier {
            if area_tier > max_tier {
                continue;
            }
        }
        
        // Check reputation requirements
        if *player_rep < item.required_reputation {
            continue;
        }
        
        // Check faction exclusivity
        if let Some(required_faction) = &item.faction_exclusive {
            if player_faction != Some(required_faction) {
                continue;
            }
        }
        
        // Calculate final price and stock
        let final_price = (item.base_price as f32 * rep_modifier.price_multiplier) as u32;
        let final_stock = if item.stock == -1 {
            -1
        } else {
            item.stock + rep_modifier.stock_bonus
        };
        
        available_items.push(AvailableTradeItem {
            item_id: item.item_id.clone(),
            price: final_price,
            stock: final_stock,
            base_price: item.base_price,
        });
    }
    
    // Add reputation-exclusive items
    for exclusive_item in &rep_modifier.exclusive_items {
        if !available_items.iter().any(|item| item.item_id == *exclusive_item) {
            available_items.push(AvailableTradeItem {
                item_id: exclusive_item.clone(),
                price: 100, // Default price for exclusive items
                stock: 1,
                base_price: 100,
            });
        }
    }
    
    let can_sell_to = *player_rep >= -25; // Can't sell to hostile traders
    let sell_multiplier = match *player_rep {
        i32::MIN..=-50 => 0.3,
        -49..=-25 => 0.5,
        -24..=0 => 0.7,
        1..=25 => 0.8,
        26..=50 => 0.9,
        _ => 1.0,
    };
    
    Some(TradeInterface {
        trader_id: trader_id.to_string(),
        trader_name: trader.name.clone(),
        available_items,
        can_sell_to,
        sell_price_multiplier: sell_multiplier,
    })
}

/// Execute a trade transaction
pub fn execute_trade(
    trade_interface: &mut TradeInterface,
    item_id: &str,
    quantity: u32,
    player_currency: &mut u32,
    player_inventory: &mut Vec<String>,
) -> Result<String, String> {
    let item = trade_interface.available_items.iter_mut()
        .find(|item| item.item_id == item_id)
        .ok_or("Item not available")?;
    
    let total_cost = item.price * quantity;
    
    if *player_currency < total_cost {
        return Err("Insufficient currency".to_string());
    }
    
    if item.stock != -1 && item.stock < quantity as i32 {
        return Err("Insufficient stock".to_string());
    }
    
    // Execute transaction
    *player_currency -= total_cost;
    for _ in 0..quantity {
        player_inventory.push(item_id.to_string());
    }
    
    if item.stock != -1 {
        item.stock -= quantity as i32;
    }
    
    Ok(format!("Purchased {} x{} for {} salt scrip", item_id, quantity, total_cost))
}

/// Sell item to trader
pub fn execute_sell(
    trade_interface: &TradeInterface,
    item_id: &str,
    quantity: u32,
    player_currency: &mut u32,
    player_inventory: &mut Vec<String>,
) -> Result<String, String> {
    if !trade_interface.can_sell_to {
        return Err("Trader refuses to buy from you".to_string());
    }
    
    let item_count = player_inventory.iter().filter(|id| *id == item_id).count() as u32;
    if item_count < quantity {
        return Err("You don't have enough of that item".to_string());
    }
    
    // Get base item value (simplified - could be from item definitions)
    let base_value = match item_id {
        "glass_shard" => 5,
        "salt_crystal" => 10,
        "cloth_scrap" => 3,
        "metal_wire" => 15,
        _ => 8, // Default value
    };
    
    let sell_price = (base_value as f32 * trade_interface.sell_price_multiplier) as u32;
    let total_value = sell_price * quantity;
    
    // Remove items from inventory
    let mut removed = 0;
    player_inventory.retain(|id| {
        if id == item_id && removed < quantity {
            removed += 1;
            false
        } else {
            true
        }
    });
    
    *player_currency += total_value;
    
    Ok(format!("Sold {} x{} for {} salt scrip", item_id, quantity, total_value))
}
