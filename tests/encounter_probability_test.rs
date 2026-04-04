// Quick test to check encounter probabilities for seed 12345
// Run with: cargo test --lib encounter_probability_test -- --nocapture

#[cfg(test)]
mod encounter_probability_test {
    use saltglass_steppe::game::encounter::should_trigger_encounter;

    #[test]
    fn test_encounter_rates_seed_12345() {
        let world_seed = 12345u64;
        let mut encounter_count = 0;
        let mut total_checks = 0;

        println!("\n=== Encounter Probability Test (Seed: 12345) ===\n");

        // Test different danger levels
        for danger_level in [1, 3, 5, 7, 10, 15] {
            let mut encounters_at_level = 0;
            let checks = 1000;

            for i in 0..checks {
                // Simulate traveling to different tiles
                let x = (i % 192) as usize;
                let y = (i / 192) as usize;
                let total_traveled = i as u64;

                if should_trigger_encounter(
                    world_seed,
                    x,
                    y,
                    total_traveled,
                    danger_level,
                    0,    // last_encounter_turn
                    1000, // current_turn (well past cooldown)
                    0,    // wayfaring_level (baseline test)
                ) {
                    encounters_at_level += 1;
                    encounter_count += 1;
                }
                total_checks += 1;
            }

            let rate = (encounters_at_level as f32 / checks as f32) * 100.0;
            println!(
                "Danger Level {:2}: {:.1}% encounter rate ({}/{})",
                danger_level, rate, encounters_at_level, checks
            );
        }

        let overall_rate = (encounter_count as f32 / total_checks as f32) * 100.0;
        println!(
            "\nOverall: {:.1}% encounter rate ({}/{})",
            overall_rate, encounter_count, total_checks
        );

        // Calculate expected rates based on config
        println!("\n=== Expected Rates (from config) ===");
        println!("Base rate: 15%");
        println!("Danger scaling: 0.02 per danger level");
        println!("Formula: base_rate * (1 + danger * scaling)");
        println!("Min: 5%, Max: 40%");
        println!("\nExpected rates:");
        for danger in [1, 3, 5, 7, 10, 15] {
            let danger_mod = 1.0 + (danger as f32 * 0.02);
            let expected = (0.15 * danger_mod).clamp(0.05, 0.40) * 100.0;
            println!("  Danger {:2}: {:.1}%", danger, expected);
        }
    }
}
