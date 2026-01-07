mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    wave_function_collapse_algorithm_test_suite,
    &[
        "wave_function_collapse_basic.json",
        "wave_function_collapse_variant.json",
    ],
    "TEST_REPORT_WAVE_FUNCTION_COLLAPSE.md",
    "wave_function_collapse Algorithm"
);
