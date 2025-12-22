use tui_rpg::des::run_scenario;
use std::fs::File;
use std::io::Write;

#[test]
fn debug_status_multiple() {
    let result = run_scenario("tests/scenarios/status_multiple.json")
        .expect("Failed to run scenario");
    
    let mut file = File::create("debug_status_log.txt").unwrap();
    
    if !result.success {
        writeln!(file, "Logs:").unwrap();
        for log in &result.logs {
            writeln!(file, "Turn {}: {}", log.turn, log.message).unwrap();
        }
        
        if let Some(state) = &result.final_state {
            writeln!(file, "Game Messages:").unwrap();
            for msg in &state.messages {
                writeln!(file, "{}", msg.text).unwrap();
            }
        }

        writeln!(file, "Assertions:").unwrap();
        for res in &result.assertion_results {
            writeln!(file, "Passed: {}, Check: {}, Message: {:?}", res.passed, res.check, res.message).unwrap();
        }
    }
    
    assert!(result.success);
}
