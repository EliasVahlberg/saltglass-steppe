mod algorithm_test_framework;
use algorithm_test_framework::*;

// Individual Algorithm Test Suites
algorithm_test_suite!(
    drunkard_walk_algorithm_test_suite,
    &[
        "drunkard_walk_basic.json",
        "drunkard_walk_variant.json",
    ],
    "TEST_REPORT_DRUNKARD_WALK.md",
    "Drunkard's Walk Algorithm"
);

algorithm_test_suite!(
    simple_rooms_algorithm_test_suite,
    &[
        "simple_rooms_basic.json",
        "simple_rooms_variant.json",
    ],
    "TEST_REPORT_SIMPLE_ROOMS.md",
    "Simple Rooms Algorithm"
);

algorithm_test_suite!(
    maze_algorithm_test_suite,
    &[
        "maze_basic.json",
        "maze_variant.json",
    ],
    "TEST_REPORT_MAZE.md",
    "Maze Generation Algorithms"
);

algorithm_test_suite!(
    voronoi_algorithm_test_suite,
    &[
        "voronoi_basic.json",
        "voronoi_variant.json",
    ],
    "TEST_REPORT_VORONOI.md",
    "Voronoi Diagram Algorithm"
);

algorithm_test_suite!(
    wave_function_collapse_test_suite,
    &[
        "wave_function_collapse_basic.json",
        "wave_function_collapse_variant.json",
    ],
    "TEST_REPORT_WAVE_FUNCTION_COLLAPSE.md",
    "Wave Function Collapse Algorithm"
);

// Hybrid Algorithm Test Suites
algorithm_test_suite!(
    hybrid_algorithms_test_suite,
    &[
        "hybrid_bsp_cellular.json",
        "hybrid_voronoi_drunkard.json",
    ],
    "TEST_REPORT_HYBRID_ALGORITHMS.md",
    "Hybrid Algorithm Combinations"
);

algorithm_test_suite!(
    sequential_algorithms_test_suite,
    &[
        "sequential_noise_maze_rooms.json",
    ],
    "TEST_REPORT_SEQUENTIAL_ALGORITHMS.md",
    "Sequential Multi-Algorithm Generation"
);

// Parameter Variation Test Suites
algorithm_test_suite!(
    parameter_variations_test_suite,
    &[
        "cellular_sparse.json",
        "cellular_dense.json",
        "bsp_small_dense.json",
        "bsp_large_sparse.json",
    ],
    "TEST_REPORT_PARAMETER_VARIATIONS.md",
    "Algorithm Parameter Variations"
);

// Comprehensive Test Suite
algorithm_test_suite!(
    comprehensive_algorithm_test_suite,
    &[
        // Basic algorithms
        "drunkard_walk_basic.json",
        "simple_rooms_basic.json",
        "maze_basic.json",
        "voronoi_basic.json",
        "wave_function_collapse_basic.json",
        
        // Hybrid approaches
        "hybrid_bsp_cellular.json",
        "hybrid_voronoi_drunkard.json",
        "sequential_noise_maze_rooms.json",
        
        // Parameter variations
        "cellular_sparse.json",
        "cellular_dense.json",
        "bsp_small_dense.json",
        "bsp_large_sparse.json",
        
        // Existing algorithms
        "bsp_basic.json",
        "cellular_caves.json",
        "saltflat_basic.json",
    ],
    "TEST_REPORT_COMPREHENSIVE.md",
    "Comprehensive Algorithm Test Suite"
);

// Category-specific test suites
algorithm_test_suite!(
    room_based_algorithms_test_suite,
    &[
        "bsp_basic.json",
        "bsp_small_dense.json",
        "bsp_large_sparse.json",
        "simple_rooms_basic.json",
        "maze_basic.json",
    ],
    "TEST_REPORT_ROOM_BASED.md",
    "Room-Based Generation Algorithms"
);

algorithm_test_suite!(
    organic_algorithms_test_suite,
    &[
        "cellular_caves.json",
        "cellular_sparse.json",
        "cellular_dense.json",
        "drunkard_walk_basic.json",
        "drunkard_walk_variant.json",
        "voronoi_basic.json",
    ],
    "TEST_REPORT_ORGANIC.md",
    "Organic/Cave Generation Algorithms"
);

algorithm_test_suite!(
    advanced_algorithms_test_suite,
    &[
        "wave_function_collapse_basic.json",
        "hybrid_bsp_cellular.json",
        "hybrid_voronoi_drunkard.json",
        "sequential_noise_maze_rooms.json",
    ],
    "TEST_REPORT_ADVANCED.md",
    "Advanced and Hybrid Algorithms"
);
