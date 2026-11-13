struct CursorUniform {
    eye_frame: u32,
    eye_motion: vec2<f32>,
}

struct CursorInput {
    @location(2) uv: vec2<f32>,
}

@group(2) @binding(0) var<uniform> cursor: CursorUniform;

const LENS_GLASS_BOUNDARY: f32 = 21.3;

fn srgb_to_linear_channel(channel: f32) -> f32 {
    if channel <= 0.04045 {
        return channel / 12.92;
    }
    return pow((channel + 0.055) / 1.055, 2.4);
}

fn srgb_color(rgb: vec3<f32>, alpha: f32) -> vec4<f32> {
    return vec4<f32>(
        srgb_to_linear_channel(rgb.r / 255.0),
        srgb_to_linear_channel(rgb.g / 255.0),
        srgb_to_linear_channel(rgb.b / 255.0),
        alpha,
    );
}

fn edge_coverage(distance: f32) -> f32 {
    let antialias = max(fwidth(distance), 0.65);
    return 1.0 - smoothstep(-0.5 * antialias, 0.5 * antialias, distance);
}

fn circle_distance(point: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(point - center) - radius;
}

fn ellipse_distance(
    point: vec2<f32>,
    center: vec2<f32>,
    radii: vec2<f32>,
) -> f32 {
    return (length((point - center) / radii) - 1.0) * min(radii.x, radii.y);
}

fn segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let segment = end - start;
    let fraction = clamp(dot(point - start, segment) / dot(segment, segment), 0.0, 1.0);
    return length(point - (start + segment * fraction));
}

fn boundary_transition(value: f32, boundary: f32, width: f32) -> f32 {
    return smoothstep(boundary - 0.5 * width, boundary + 0.5 * width, value);
}

fn band_coverage(value: f32, inner: f32, outer: f32, width: f32) -> f32 {
    return boundary_transition(value, inner, width)
        * (1.0 - boundary_transition(value, outer, width));
}

fn over(destination: vec4<f32>, source: vec4<f32>, coverage: f32) -> vec4<f32> {
    let alpha = clamp(source.a * coverage, 0.0, 1.0);
    return vec4<f32>(
        source.rgb * alpha + destination.rgb * (1.0 - alpha),
        alpha + destination.a * (1.0 - alpha),
    );
}

fn over_premultiplied(destination: vec4<f32>, source: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        source.rgb + destination.rgb * (1.0 - source.a),
        source.a + destination.a * (1.0 - source.a),
    );
}

fn finish_color(premultiplied: vec4<f32>) -> vec4<f32> {
    if premultiplied.a <= 0.0001 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(premultiplied.rgb / premultiplied.a, premultiplied.a);
}

fn magnifier_color(point: vec2<f32>) -> vec4<f32> {
    let outline = srgb_color(vec3<f32>(107.0, 79.0, 18.0), 1.0);
    let shadow = srgb_color(vec3<f32>(78.0, 55.0, 19.0), 190.0 / 255.0);
    let gold = srgb_color(vec3<f32>(212.0, 160.0, 74.0), 1.0);
    let gold_light = srgb_color(vec3<f32>(253.0, 216.0, 104.0), 1.0);
    let gold_dark = srgb_color(vec3<f32>(139.0, 102.0, 28.0), 1.0);
    let rust = srgb_color(vec3<f32>(196.0, 66.0, 31.0), 1.0);
    let glass = srgb_color(vec3<f32>(224.0, 224.0, 224.0), 1.0);
    let glass_light = srgb_color(vec3<f32>(250.0, 250.0, 250.0), 1.0);
    let glass_shade = srgb_color(vec3<f32>(190.0, 190.0, 190.0), 1.0);

    var color = vec4<f32>(0.0);
    let lens_distance = length(point - vec2<f32>(42.0, 42.0));
    let lens_coverage = edge_coverage(lens_distance - 30.5);
    let lens_antialias = max(fwidth(lens_distance), 0.65);
    let inner_glass = 1.0 - boundary_transition(
        lens_distance,
        LENS_GLASS_BOUNDARY,
        lens_antialias,
    );
    let shadow_coverage = edge_coverage(circle_distance(
        point,
        vec2<f32>(44.0, 45.0),
        31.8,
    )) * (1.0 - inner_glass);
    color = over(color, shadow, shadow_coverage);

    let handle_start = vec2<f32>(63.0, 63.0);
    let handle_end = vec2<f32>(83.0, 83.0);
    let handle_distance = segment_distance(point, handle_start, handle_end);
    color = over(color, outline, edge_coverage(handle_distance - 8.0));
    color = over(color, gold, edge_coverage(handle_distance - 6.0));

    let highlight_distance = segment_distance(
        point,
        handle_start + vec2<f32>(-1.5, 1.5),
        handle_end + vec2<f32>(-1.5, 1.5),
    );
    color = over(
        color,
        gold_light,
        edge_coverage(highlight_distance - 1.0) * edge_coverage(handle_distance - 6.0),
    );

    let rust_distance = segment_distance(
        point,
        handle_start + vec2<f32>(2.8, -2.8),
        handle_end + vec2<f32>(2.8, -2.8),
    );
    color = over(
        color,
        rust,
        edge_coverage(rust_distance - 1.1) * edge_coverage(handle_distance - 6.0),
    );

    var lens_rgb = glass.rgb;
    lens_rgb = mix(
        lens_rgb,
        outline.rgb,
        boundary_transition(lens_distance, LENS_GLASS_BOUNDARY, lens_antialias),
    );
    lens_rgb = mix(
        lens_rgb,
        gold.rgb,
        boundary_transition(lens_distance, 22.1, lens_antialias),
    );
    lens_rgb = mix(
        lens_rgb,
        gold_dark.rgb,
        boundary_transition(lens_distance, 24.0, lens_antialias),
    );
    lens_rgb = mix(
        lens_rgb,
        gold.rgb,
        boundary_transition(lens_distance, 26.5, lens_antialias),
    );
    lens_rgb = mix(
        lens_rgb,
        outline.rgb,
        boundary_transition(lens_distance, 28.5, lens_antialias),
    );

    let outer_gold_band = band_coverage(lens_distance, 26.5, 28.5, lens_antialias);
    lens_rgb = mix(
        lens_rgb,
        gold_light.rgb,
        outer_gold_band * edge_coverage(ellipse_distance(
            point,
            vec2<f32>(34.0, 28.0),
            vec2<f32>(12.0, 3.5),
        )),
    );
    lens_rgb = mix(
        lens_rgb,
        rust.rgb,
        outer_gold_band * edge_coverage(ellipse_distance(
            point,
            vec2<f32>(50.0, 48.0),
            vec2<f32>(9.0, 5.0),
        )),
    );

    let inner_gold_band = band_coverage(lens_distance, 22.1, 24.0, lens_antialias);
    lens_rgb = mix(
        lens_rgb,
        gold_light.rgb,
        inner_gold_band * edge_coverage(ellipse_distance(
            point,
            vec2<f32>(34.0, 28.0),
            vec2<f32>(10.0, 2.8),
        )),
    );

    lens_rgb = mix(
        lens_rgb,
        glass_light.rgb,
        inner_glass * edge_coverage(ellipse_distance(
            point,
            vec2<f32>(34.0, 32.0),
            vec2<f32>(10.0, 3.0),
        )),
    );
    lens_rgb = mix(
        lens_rgb,
        glass_shade.rgb,
        inner_glass * edge_coverage(ellipse_distance(
            point,
            vec2<f32>(49.0, 51.0),
            vec2<f32>(13.0, 8.0),
        )),
    );

    let glass_alpha = mix(0.94, 0.30, inner_glass);
    color = over(color, vec4<f32>(lens_rgb, glass_alpha), lens_coverage);
    return color;
}

fn closed_eye_color(
    point: vec2<f32>,
    outline: vec4<f32>,
    center_y: f32,
    lash_start_y: f32,
    stroke_width: f32,
    vertical_offset: f32,
) -> vec4<f32> {
    let dx = point.x - 42.0;
    let offset = vec2<f32>(0.0, vertical_offset);
    let eyelid_y = dx * dx * 0.014 + center_y + vertical_offset;
    let eyelid = edge_coverage(abs(point.y - eyelid_y) - stroke_width)
        * edge_coverage(abs(dx) - 17.5);
    var color = vec4<f32>(0.0);
    color = over(color, outline, eyelid);
    color = over(
        color,
        outline,
        edge_coverage(segment_distance(
            point,
            vec2<f32>(26.0, lash_start_y) + offset,
            vec2<f32>(22.5, lash_start_y + 3.0) + offset,
        ) - 0.85),
    );
    color = over(
        color,
        outline,
        edge_coverage(segment_distance(
            point,
            vec2<f32>(58.0, lash_start_y) + offset,
            vec2<f32>(61.5, lash_start_y + 3.0) + offset,
        ) - 0.85),
    );
    return color;
}

fn eye_color(point: vec2<f32>, frame: u32, motion: vec2<f32>) -> vec4<f32> {
    let outline = srgb_color(vec3<f32>(107.0, 79.0, 18.0), 1.0);
    let sclera = srgb_color(vec3<f32>(255.0, 248.0, 226.0), 1.0);
    let iris = srgb_color(vec3<f32>(190.0, 115.0, 44.0), 1.0);
    let iris_light = srgb_color(vec3<f32>(226.0, 157.0, 65.0), 1.0);
    let pupil = srgb_color(vec3<f32>(61.0, 40.0, 20.0), 1.0);
    let glint = srgb_color(vec3<f32>(255.0, 255.0, 242.0), 1.0);

    if frame == 3u {
        return closed_eye_color(point, outline, 40.0, 43.0, 1.35, motion.y);
    }

    if frame == 2u {
        return closed_eye_color(point, outline, 42.0, 44.0, 1.25, 0.0);
    }

    var eye_height = 12.5;
    var iris_height = 8.1;
    var pupil_height = 5.4;
    if frame == 1u {
        eye_height = 7.0;
        iris_height = 4.6;
        pupil_height = 3.5;
    }

    let outer_eye = edge_coverage(ellipse_distance(
        point,
        vec2<f32>(42.0, 42.0),
        vec2<f32>(19.0, eye_height + 1.5),
    ));
    let sclera_coverage = edge_coverage(ellipse_distance(
        point,
        vec2<f32>(42.0, 42.0),
        vec2<f32>(17.0, eye_height - 1.0),
    ));
    var color = vec4<f32>(0.0);
    color = over(color, outline, max(outer_eye - sclera_coverage, 0.0));
    color = over(color, sclera, sclera_coverage);

    var gaze_center = vec2<f32>(42.0, 43.0);
    if frame == 4u {
        gaze_center += vec2<f32>(motion.x, 4.0);
    }

    let iris_outer = edge_coverage(ellipse_distance(
        point,
        gaze_center,
        vec2<f32>(7.4, iris_height + 1.2),
    )) * sclera_coverage;
    let iris_inner = edge_coverage(ellipse_distance(
        point,
        gaze_center,
        vec2<f32>(6.0, iris_height),
    )) * sclera_coverage;
    color = over(color, outline, max(iris_outer - iris_inner, 0.0));
    color = over(color, iris, iris_inner);

    let iris_light_coverage = iris_inner * edge_coverage(ellipse_distance(
        point,
        gaze_center + vec2<f32>(-1.5, -2.5),
        vec2<f32>(4.3, 4.2),
    ));
    color = over(color, iris_light, iris_light_coverage);

    let pupil_coverage = iris_inner * edge_coverage(ellipse_distance(
        point,
        gaze_center + vec2<f32>(1.0, 0.2),
        vec2<f32>(3.4, pupil_height),
    ));
    color = over(color, pupil, pupil_coverage);

    let first_glint = edge_coverage(ellipse_distance(
        point,
        gaze_center + vec2<f32>(-2.6, -3.3),
        vec2<f32>(2.0, 2.4),
    ));
    let second_glint = edge_coverage(ellipse_distance(
        point,
        gaze_center + vec2<f32>(3.0, 2.0),
        vec2<f32>(0.9, 1.0),
    ));
    let glint_coverage = min(iris_inner, max(first_glint, second_glint))
        * (1.0 - pupil_coverage);
    color = over(color, glint, glint_coverage);

    return color;
}

@fragment
fn cursor_mesh_fragment(input: CursorInput) -> @location(0) vec4<f32> {
    let point = input.uv * 96.0;
    let magnifier = magnifier_color(point);
    let eye = eye_color(point, cursor.eye_frame, cursor.eye_motion);
    return finish_color(over_premultiplied(magnifier, eye));
}
