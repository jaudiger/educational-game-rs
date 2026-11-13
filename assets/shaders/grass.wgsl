struct GrassInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct GrassUniform {
    primary: vec4<f32>,
    secondary: vec4<f32>,
    motion: vec4<f32>,
}

@group(1) @binding(0) var<uniform> grass: GrassUniform;

fn capsule_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>, radius: f32) -> f32 {
    let segment = end - start;
    let denominator = max(dot(segment, segment), 0.000001);
    let along = clamp(dot(point - start, segment) / denominator, 0.0, 1.0);
    return length(point - (start + segment * along)) - radius;
}

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.0001);
    return 1.0 - smoothstep(-width, width, distance);
}

fn grass_fragment(point: vec2<f32>) -> vec4<f32> {
    let first_blade = capsule_distance(point, vec2<f32>(-0.38, -0.50), vec2<f32>(-0.47, 0.20), 0.014);
    let second_blade = capsule_distance(point, vec2<f32>(-0.27, -0.50), vec2<f32>(-0.22, 0.38), 0.013);
    let third_blade = capsule_distance(point, vec2<f32>(-0.16, -0.50), vec2<f32>(-0.06, 0.16), 0.018);
    let fourth_blade = capsule_distance(point, vec2<f32>(-0.05, -0.50), vec2<f32>(-0.12, 0.33), 0.014);
    let fifth_blade = capsule_distance(point, vec2<f32>(0.08, -0.50), vec2<f32>(0.02, 0.25), 0.016);
    let sixth_blade = capsule_distance(point, vec2<f32>(0.20, -0.50), vec2<f32>(0.28, 0.39), 0.013);
    let seventh_blade = capsule_distance(point, vec2<f32>(0.32, -0.50), vec2<f32>(0.44, 0.16), 0.017);
    let eighth_blade = capsule_distance(point, vec2<f32>(0.42, -0.50), vec2<f32>(0.34, 0.24), 0.012);
    let blades = min(
        min(min(first_blade, second_blade), min(third_blade, fourth_blade)),
        min(min(fifth_blade, sixth_blade), min(seventh_blade, eighth_blade)),
    );
    let shade = smoothstep(-0.45, 0.38, point.y) * 0.35;
    let color = mix(grass.primary.rgb, grass.secondary.rgb, shade);
    return vec4<f32>(color, coverage(blades) * grass.primary.a);
}

@fragment
fn fragment(input: GrassInput) -> @location(0) vec4<f32> {
    let aspect = input.size.x / max(input.size.y, 0.001);
    let point = vec2<f32>((input.uv.x - 0.5) * aspect, 0.5 - input.uv.y);
    let height = clamp(point.y + 0.5, 0.0, 1.0);
    let bent_point = vec2<f32>(point.x - grass.motion.x * height * height, point.y);
    return grass_fragment(bent_point);
}
