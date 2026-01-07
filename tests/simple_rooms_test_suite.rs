mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    simple_rooms_algorithm_test_suite,
    &[
        "simple_rooms_basic.json",
        "simple_rooms_variant.json",
    ],
    "TEST_REPORT_SIMPLE_ROOMS.md",
    "simple_rooms Algorithm"
);
