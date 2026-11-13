const SHADERS: &[(&str, &str, &[&str])] = &[
    (
        "ui_shape.wgsl",
        include_str!("../assets/shaders/ui_shape.wgsl"),
        &["fragment"],
    ),
    (
        "landscape.wgsl",
        include_str!("../assets/shaders/landscape.wgsl"),
        &["fragment"],
    ),
    (
        "tree.wgsl",
        include_str!("../assets/shaders/tree.wgsl"),
        &["fragment"],
    ),
    (
        "flower.wgsl",
        include_str!("../assets/shaders/flower.wgsl"),
        &["fragment"],
    ),
    (
        "grass.wgsl",
        include_str!("../assets/shaders/grass.wgsl"),
        &["fragment"],
    ),
    (
        "stone.wgsl",
        include_str!("../assets/shaders/stone.wgsl"),
        &["fragment"],
    ),
    (
        "balloon.wgsl",
        include_str!("../assets/shaders/balloon.wgsl"),
        &["balloon_mesh_fragment", "balloon_ui_fragment"],
    ),
    (
        "cloud.wgsl",
        include_str!("../assets/shaders/cloud.wgsl"),
        &["cloud_mesh_fragment", "cloud_ui_fragment"],
    ),
    (
        "cursor.wgsl",
        include_str!("../assets/shaders/cursor.wgsl"),
        &["cursor_mesh_fragment"],
    ),
    (
        "sun.wgsl",
        include_str!("../assets/shaders/sun.wgsl"),
        &["fragment"],
    ),
];

#[test]
fn current_wgsl_shaders_parse_and_validate() {
    for (path, source, entry_points) in SHADERS {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{path}: {error:?}"));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|error| panic!("{path}: {error:?}"));

        for entry_point in *entry_points {
            assert!(
                module
                    .entry_points
                    .iter()
                    .any(|entry| entry.name == *entry_point),
                "{path} is missing entry point {entry_point}"
            );
        }
    }
}
