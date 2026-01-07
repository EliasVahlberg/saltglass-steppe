mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    drunkard_walk_algorithm_test_suite,
    &[
        "drunkard_walk_basic.json",
        "drunkard_walk_variant.json",
    ],
    "TEST_REPORT_DRUNKARD_WALK.md",
    "drunkard_walk Algorithm"
);
