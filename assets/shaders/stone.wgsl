struct StoneInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct StoneUniform {
    primary: vec4<f32>,
    secondary: vec4<f32>,
    accent: vec4<f32>,
}

@group(1) @binding(0) var<uniform> stone: StoneUniform;

fn stone_distance(point: vec2<f32>) -> f32 {
    let flattened = point * vec2<f32>(1.0, 1.7);
    let angle = atan2(flattened.y, flattened.x);
    let radius = 0.54 + 0.055 * cos(angle * 3.0) + 0.025 * sin(angle * 5.0 + 0.4);
    return length(flattened) - radius;
}

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.0001);
    return 1.0 - smoothstep(-width, width, distance);
}

fn over(bottom: vec4<f32>, top: vec4<f32>) -> vec4<f32> {
    let alpha = top.a + bottom.a * (1.0 - top.a);
    let rgb = (top.rgb * top.a + bottom.rgb * bottom.a * (1.0 - top.a)) / max(alpha, 0.0001);
    return vec4<f32>(rgb, alpha);
}

fn stone_fragment(point: vec2<f32>) -> vec4<f32> {
    let shadow_distance = stone_distance(point - vec2<f32>(0.0, -0.09)) - 0.025;
    let body_distance = stone_distance(point);
    let highlight_point = (point - vec2<f32>(-0.12, 0.11)) * vec2<f32>(1.0, 1.7);
    let highlight_distance = length(highlight_point) - 0.17;
    let light = smoothstep(-0.22, 0.25, point.y) * 0.42;
    let body = mix(stone.primary.rgb, stone.secondary.rgb, light);

    var result = vec4<f32>(0.0);
    result = over(result, vec4<f32>(stone.accent.rgb, coverage(shadow_distance) * stone.accent.a));
    result = over(result, vec4<f32>(body, coverage(body_distance) * stone.primary.a));
    result = over(result, vec4<f32>(stone.secondary.rgb, coverage(highlight_distance) * stone.secondary.a * 0.32));
    return result;
}

@fragment
fn fragment(input: StoneInput) -> @location(0) vec4<f32> {
    let aspect = input.size.x / max(input.size.y, 0.001);
    let point = vec2<f32>((input.uv.x - 0.5) * aspect, 0.5 - input.uv.y);
    return stone_fragment(point);
}
