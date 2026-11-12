struct BalloonUniform {
    part_id: u32,
    flame_intensity: f32,
    cord_endpoints: array<vec4<f32>, 2>,
}

struct MeshBalloonInput {
    @location(2) uv: vec2<f32>,
}

struct UiBalloonInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

const BALLOON_ENVELOPE: u32 = 0u;
const BALLOON_BASKET: u32 = 1u;
const BALLOON_FLAME: u32 = 2u;
const BASKET_OUTER_MAX_HALF_WIDTH: f32 = 54.0;
const BASKET_OUTER_TOP_HALF_WIDTH: f32 = 37.0;
const BASKET_OUTER_HALF_HEIGHT: f32 = 34.0;
const BASKET_CORNER_RADIUS: f32 = 10.0;
const PI: f32 = 3.141592653589793;

@group(2) @binding(0) var<uniform> mesh_balloon: BalloonUniform;
@group(1) @binding(0) var<uniform> ui_balloon: BalloonUniform;

fn srgb_channel_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        return channel / 12.92;
    }
    return pow((channel + 0.055) / 1.055, 2.4);
}

fn color_from_srgb(rgb: vec3<f32>) -> vec3<f32> {
    let normalized = rgb / 255.0;
    return vec3<f32>(
        srgb_channel_to_linear(normalized.r),
        srgb_channel_to_linear(normalized.g),
        srgb_channel_to_linear(normalized.b),
    );
}

fn edge_coverage(distance: f32) -> f32 {
    let antialias = max(fwidth(distance), 0.65);
    return 1.0 - smoothstep(-0.5 * antialias, 0.5 * antialias, distance);
}

fn envelope_half_width(y: f32) -> f32 {
    let progress = clamp((y - 8.0) / 254.0, 0.0, 1.0);
    let oval_profile = max(sin(PI * progress), 0.0);
    let taper = mix(0.48, 0.42, progress);
    return 116.0 * pow(oval_profile, taper);
}

fn envelope_distance(point: vec2<f32>) -> f32 {
    let half_width = envelope_half_width(point.y);
    let horizontal = abs(point.x - 128.0) - half_width;
    let vertical = max(8.0 - point.y, point.y - 262.0);
    return max(horizontal, vertical);
}

fn segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let segment = end - start;
    let fraction = clamp(dot(point - start, segment) / dot(segment, segment), 0.0, 1.0);
    return length(point - (start + segment * fraction));
}

fn capsule_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>, radius: f32) -> f32 {
    return segment_distance(point, start, end) - radius;
}


fn cord_sag(start: vec2<f32>, end: vec2<f32>) -> f32 {
    let rest_length = 62.0;
    let slack = clamp(1.0 - length(end - start) / rest_length, 0.0, 1.0);
    return slack * 5.0;
}

fn curved_cord_distance(
    point: vec2<f32>,
    start: vec2<f32>,
    end: vec2<f32>,
    bend: f32,
) -> f32 {
    var distance = 10000.0;
    var previous = start;
    for (var index = 1u; index <= 6u; index += 1u) {
        let fraction = f32(index) / 6.0;
        let curve = start + (end - start) * fraction
            + vec2<f32>(0.0, bend * 4.0 * fraction * (1.0 - fraction));
        distance = min(distance, segment_distance(point, previous, curve));
        previous = curve;
    }
    return distance;
}

fn envelope_color(point: vec2<f32>, cord_endpoints: array<vec4<f32>, 2>) -> vec4<f32> {
    let body_distance = envelope_distance(point);
    let body_coverage = edge_coverage(body_distance);
    let inner_coverage = edge_coverage(body_distance + 5.0);
    let outline = color_from_srgb(vec3<f32>(107.0, 79.0, 18.0));
    let red_panel = color_from_srgb(vec3<f32>(196.0, 66.0, 31.0));
    let orange_panel = color_from_srgb(vec3<f32>(232.0, 93.0, 58.0));
    let gold = color_from_srgb(vec3<f32>(246.0, 177.0, 42.0));
    let yellow_center = color_from_srgb(vec3<f32>(255.0, 204.0, 48.0));
    let gold_light = color_from_srgb(vec3<f32>(255.0, 229.0, 103.0));
    let gold_shadow = color_from_srgb(vec3<f32>(228.0, 145.0, 34.0));

    let half_width = max(envelope_half_width(point.y), 0.001);
    let panel_position = (point.x - 128.0) / half_width;
    let side_position = abs(panel_position);
    let center_panel = 1.0 - smoothstep(0.28, 0.72, side_position);
    let side_panel = smoothstep(0.56, 0.76, side_position);
    let outer_panel = smoothstep(0.80, 0.98, side_position);
    var fabric = mix(gold, yellow_center, center_panel * 0.92);
    fabric = mix(fabric, orange_panel, side_panel);
    fabric = mix(fabric, red_panel, outer_panel * 0.72);

    let vertical_shade = smoothstep(172.0, 254.0, point.y) * 0.22;
    let top_glow = (1.0 - smoothstep(18.0, 190.0, point.y)) * 0.12;
    fabric = mix(fabric, gold_shadow, vertical_shade);
    fabric = mix(fabric, gold_light, top_glow * (1.0 - side_panel * 0.55));

    let center_highlight = (1.0 - smoothstep(0.08, 0.48, side_position))
        * (1.0 - vertical_shade)
        * 0.24;
    fabric = mix(fabric, gold_light, center_highlight);

    let seam_distance = min(
        min(abs(panel_position - 0.38), abs(panel_position + 0.38)),
        min(abs(panel_position - 0.72), abs(panel_position + 0.72)),
    );
    let seam_width = max(fwidth(panel_position) * 2.0, 0.018);
    let seam = 1.0 - smoothstep(0.0, seam_width, seam_distance);
    fabric *= 1.0 - seam * 0.14;

    let body_color = mix(outline, fabric, inner_coverage);
    let rope_color = color_from_srgb(vec3<f32>(107.0, 79.0, 18.0));
    let rope_distance = min(
        min(
            curved_cord_distance(
                point,
                vec2<f32>(94.0, 257.0),
                cord_endpoints[0].xy,
                cord_sag(vec2<f32>(94.0, 257.0), cord_endpoints[0].xy),
            ) - 1.7,
            curved_cord_distance(
                point,
                vec2<f32>(107.0, 258.0),
                cord_endpoints[0].zw,
                cord_sag(vec2<f32>(107.0, 258.0), cord_endpoints[0].zw),
            ) - 1.5,
        ),
        min(
            curved_cord_distance(
                point,
                vec2<f32>(149.0, 258.0),
                cord_endpoints[1].xy,
                cord_sag(vec2<f32>(149.0, 258.0), cord_endpoints[1].xy),
            ) - 1.5,
            curved_cord_distance(
                point,
                vec2<f32>(162.0, 257.0),
                cord_endpoints[1].zw,
                cord_sag(vec2<f32>(162.0, 257.0), cord_endpoints[1].zw),
            ) - 1.7,
        ),
    );
    let rope_coverage = edge_coverage(rope_distance);
    let alpha = body_coverage + rope_coverage * (1.0 - body_coverage);
    let premultiplied = body_color * body_coverage
        + rope_color * rope_coverage * (1.0 - body_coverage);
    if alpha <= 0.0001 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(premultiplied / alpha, alpha);
}

fn trapezoid_distance(point: vec2<f32>, bottom_half_width: f32, top_half_width: f32, half_height: f32) -> f32 {
    let p = vec2<f32>(abs(point.x), point.y);
    let k1 = vec2<f32>(top_half_width, half_height);
    let k2 = vec2<f32>(top_half_width - bottom_half_width, 2.0 * half_height);
    let selected_width = select(bottom_half_width, top_half_width, p.y > 0.0);
    let ca = vec2<f32>(p.x - min(p.x, selected_width), abs(p.y) - half_height);
    let cb = p - k1 + k2 * clamp(dot(k1 - p, k2) / dot(k2, k2), 0.0, 1.0);
    var sign = 1.0;
    if cb.x < 0.0 && ca.y < 0.0 {
        sign = -1.0;
    }
    return sign * sqrt(min(dot(ca, ca), dot(cb, cb)));
}

fn rounded_trapezoid_distance(
    point: vec2<f32>,
    outer_max_half_width: f32,
    outer_top_half_width: f32,
    outer_half_height: f32,
    corner_radius: f32,
) -> f32 {
    let inner_bottom_half_width = outer_max_half_width - corner_radius;
    let inner_half_height = outer_half_height - corner_radius;
    return trapezoid_distance(
        point,
        inner_bottom_half_width,
        outer_top_half_width,
        inner_half_height,
    ) - corner_radius;
}

fn rounded_box_distance(
    point: vec2<f32>,
    center: vec2<f32>,
    half_size: vec2<f32>,
    radius: f32,
) -> f32 {
    let q = abs(point - center) - half_size + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

fn ellipse_distance(point: vec2<f32>, center: vec2<f32>, radii: vec2<f32>) -> f32 {
    return (length((point - center) / radii) - 1.0) * min(radii.x, radii.y);
}

fn ballast_bag(
    point: vec2<f32>,
    center: vec2<f32>,
    detail_visibility: f32,
    outline: vec3<f32>,
    bag_color: vec3<f32>,
    bag_light: vec3<f32>,
) -> vec4<f32> {
    let distance = rounded_box_distance(point, center, vec2<f32>(7.0, 8.0), 5.0);
    let coverage = edge_coverage(distance);
    let fill = edge_coverage(distance + 1.5);
    let highlight = edge_coverage(ellipse_distance(
        point,
        center + vec2<f32>(-2.0, -3.0),
        vec2<f32>(2.3, 4.0),
    ));
    let tie = edge_coverage(capsule_distance(
        point,
        center + vec2<f32>(-2.5, -6.0),
        center + vec2<f32>(2.5, -6.0),
        0.8,
    ));

    var color = mix(outline, bag_color, fill);
    color = mix(color, bag_light, highlight * fill * detail_visibility * 0.55);
    color = mix(color, outline, tie * fill * detail_visibility * 0.65);
    return vec4<f32>(color, coverage);
}

fn basket_color(point: vec2<f32>) -> vec4<f32> {
    let local_point = vec2<f32>(point.x - 64.0, 42.0 - point.y);
    let distance = rounded_trapezoid_distance(
        local_point,
        BASKET_OUTER_MAX_HALF_WIDTH,
        BASKET_OUTER_TOP_HALF_WIDTH,
        BASKET_OUTER_HALF_HEIGHT,
        BASKET_CORNER_RADIUS,
    );
    let coverage = edge_coverage(distance);
    let inner_coverage = edge_coverage(distance + 4.0);
    let outline = color_from_srgb(vec3<f32>(107.0, 79.0, 18.0));
    let dark_wicker = color_from_srgb(vec3<f32>(129.0, 96.0, 23.0));
    let wicker = color_from_srgb(vec3<f32>(180.0, 132.0, 45.0));
    let wicker_light = color_from_srgb(vec3<f32>(206.0, 163.0, 87.0));

    let pixel_footprint = max(fwidth(point.x), fwidth(point.y));
    let detail_visibility = 1.0 - smoothstep(1.0, 3.5, pixel_footprint);
    let row_position = point.y - 13.0;
    let row_phase = fract(row_position / 11.0);
    let row_distance = min(row_phase, 1.0 - row_phase) * 11.0;
    let horizontal_weave = (1.0 - smoothstep(0.8, 1.8, row_distance)) * detail_visibility;
    let strand_x = point.x + sin(point.y * 0.09) * 1.5;
    let strand_phase = fract((strand_x - 15.0) / 14.0);
    let strand_distance = abs(strand_phase - 0.5) * 14.0;
    let vertical_weave = (1.0 - smoothstep(1.0, 2.4, strand_distance)) * detail_visibility;

    let base = mix(wicker, wicker_light, 0.25 + 0.18 * sin(point.y * PI / 22.0));
    let weave_color = mix(base, dark_wicker, horizontal_weave * 0.48);
    let woven = mix(weave_color, wicker_light, vertical_weave * 0.16);
    let rim = 1.0 - smoothstep(2.0, 4.0, abs(point.y - 17.0));
    let interior = mix(woven, dark_wicker, rim * 0.7);
    let bag_outline = outline;
    let bag_color = color_from_srgb(vec3<f32>(164.0, 121.0, 35.0));
    let bag_light = wicker_light;
    let bag_a = ballast_bag(point, vec2<f32>(33.0, 19.0), detail_visibility, bag_outline, bag_color, bag_light);
    let bag_b = ballast_bag(point, vec2<f32>(54.0, 19.0), detail_visibility, bag_outline, bag_color, bag_light);
    let bag_c = ballast_bag(point, vec2<f32>(74.0, 19.0), detail_visibility, bag_outline, bag_color, bag_light);
    let bag_d = ballast_bag(point, vec2<f32>(95.0, 19.0), detail_visibility, bag_outline, bag_color, bag_light);
    var basket_content = mix(interior, bag_a.rgb, bag_a.a);
    basket_content = mix(basket_content, bag_b.rgb, bag_b.a);
    basket_content = mix(basket_content, bag_c.rgb, bag_c.a);
    basket_content = mix(basket_content, bag_d.rgb, bag_d.a);
    let basket = mix(outline, basket_content, inner_coverage);
    let alpha = coverage;
    if alpha <= 0.0001 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(basket, alpha);
}

fn flame_distance(point: vec2<f32>) -> f32 {
    let progress = clamp((point.y - 5.0) / 70.0, 0.0, 1.0);
    let width = 23.0 * pow(max(sin(PI * progress), 0.0), 0.72);
    let horizontal = abs(point.x - 32.0) - width;
    let vertical = max(5.0 - point.y, point.y - 75.0);
    return max(horizontal, vertical);
}

fn flame_color(point: vec2<f32>, intensity: f32) -> vec4<f32> {
    let flame_coverage = edge_coverage(flame_distance(point));
    let core_point = vec2<f32>((point.x - 32.0) * 1.6 + 32.0, point.y);
    let core_coverage = edge_coverage(flame_distance(core_point));
    let outer = color_from_srgb(vec3<f32>(244.0, 73.0, 22.0));
    let inner = color_from_srgb(vec3<f32>(255.0, 178.0, 35.0));
    let core = color_from_srgb(vec3<f32>(255.0, 239.0, 153.0));
    let color = mix(mix(outer, inner, core_coverage), core, core_coverage * 0.7);
    let alpha = flame_coverage * intensity;
    if alpha <= 0.0001 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color, alpha);
}

fn balloon_fragment(uv: vec2<f32>, balloon: BalloonUniform) -> vec4<f32> {
    if balloon.part_id == BALLOON_ENVELOPE {
        return envelope_color(uv * vec2<f32>(256.0, 320.0), balloon.cord_endpoints);
    }
    if balloon.part_id == BALLOON_BASKET {
        return basket_color(uv * vec2<f32>(128.0, 80.0));
    }
    if balloon.part_id == BALLOON_FLAME {
        return flame_color(uv * vec2<f32>(64.0, 80.0), balloon.flame_intensity);
    }
    return vec4<f32>(0.0);
}

@fragment
fn balloon_mesh_fragment(input: MeshBalloonInput) -> @location(0) vec4<f32> {
    return balloon_fragment(input.uv, mesh_balloon);
}

@fragment
fn balloon_ui_fragment(input: UiBalloonInput) -> @location(0) vec4<f32> {
    return balloon_fragment(input.uv, ui_balloon);
}
