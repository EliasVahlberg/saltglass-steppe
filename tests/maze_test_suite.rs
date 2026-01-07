mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    maze_algorithm_test_suite,
    &[
        "maze_basic.json",
        "maze_variant.json",
    ],
    "TEST_REPORT_MAZE.md",
    "maze Algorithm"
);
