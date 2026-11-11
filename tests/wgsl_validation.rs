const SHADERS: &[(&str, &str, &[&str])] = &[
    (
        "menu_decoration.wgsl",
        include_str!("../assets/shaders/menu_decoration.wgsl"),
        &["fragment"],
    ),
    (
        "procedural_balloon.wgsl",
        include_str!("../assets/shaders/procedural_balloon.wgsl"),
        &["balloon_mesh_fragment", "balloon_ui_fragment"],
    ),
    (
        "procedural_cloud.wgsl",
        include_str!("../assets/shaders/procedural_cloud.wgsl"),
        &["cloud_mesh_fragment", "cloud_ui_fragment"],
    ),
    (
        "procedural_cursor.wgsl",
        include_str!("../assets/shaders/procedural_cursor.wgsl"),
        &["cursor_mesh_fragment"],
    ),
    (
        "summary_lens_flare.wgsl",
        include_str!("../assets/shaders/summary_lens_flare.wgsl"),
        &["fragment"],
    ),
    (
        "summary_sun.wgsl",
        include_str!("../assets/shaders/summary_sun.wgsl"),
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
