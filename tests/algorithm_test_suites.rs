mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    bsp_algorithm_test_suite,
    &[
        "bsp_small_rooms.json",
        "bsp_large_rooms.json",
    ],
    "TEST_REPORT_BSP.md",
    "BSP Algorithm"
);

algorithm_test_suite!(
    cellular_automata_test_suite,
    &[
        "cellular_caves.json", 
        "cellular_dense.json",
    ],
    "TEST_REPORT_CELLULAR_AUTOMATA.md",
    "Cellular Automata Algorithm"
);

algorithm_test_suite!(
    dungeon_generation_test_suite,
    &[
        "bsp_small_rooms.json",
        "bsp_large_rooms.json",
        "cellular_caves.json",
    ],
    "TEST_REPORT_DUNGEONS.md",
    "Dungeon Generation Algorithms"
);
