mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    voronoi_algorithm_test_suite,
    &[
        "voronoi_basic.json",
        "voronoi_variant.json",
    ],
    "TEST_REPORT_VORONOI.md",
    "voronoi Algorithm"
);
