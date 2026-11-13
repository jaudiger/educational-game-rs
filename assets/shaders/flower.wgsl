struct FlowerInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct FlowerUniform {
    primary: vec4<f32>,
    secondary: vec4<f32>,
    accent: vec4<f32>,
    motion: vec4<f32>,
}

@group(1) @binding(0) var<uniform> flower: FlowerUniform;

fn capsule_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>, radius: f32) -> f32 {
    let segment = end - start;
    let denominator = max(dot(segment, segment), 0.000001);
    let along = clamp(dot(point - start, segment) / denominator, 0.0, 1.0);
    return length(point - (start + segment * along)) - radius;
}

fn flower_distance(point: vec2<f32>, center: vec2<f32>) -> f32 {
    let offset = point - center;
    let angle = atan2(offset.y, offset.x);
    return length(offset) - (0.064 + 0.018 * cos(angle * 5.0));
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

fn flower_fragment(point: vec2<f32>) -> vec4<f32> {
    let left_stem = capsule_distance(point, vec2<f32>(-0.12, -0.50), vec2<f32>(-0.25, 0.11), 0.009);
    let middle_stem = capsule_distance(point, vec2<f32>(-0.02, -0.50), vec2<f32>(-0.01, 0.30), 0.009);
    let right_stem = capsule_distance(point, vec2<f32>(0.12, -0.50), vec2<f32>(0.22, 0.16), 0.009);
    let stems = min(left_stem, min(middle_stem, right_stem));

    let first_leaf = capsule_distance(point, vec2<f32>(-0.18, -0.19), vec2<f32>(-0.31, -0.10), 0.016);
    let second_leaf = capsule_distance(point, vec2<f32>(0.02, -0.10), vec2<f32>(0.15, -0.01), 0.016);
    let third_leaf = capsule_distance(point, vec2<f32>(0.17, -0.22), vec2<f32>(0.30, -0.13), 0.016);
    let leaves = min(first_leaf, min(second_leaf, third_leaf));

    let left_flower = flower_distance(point, vec2<f32>(-0.25, 0.11));
    let middle_flower = flower_distance(point, vec2<f32>(-0.01, 0.30));
    let right_flower = flower_distance(point, vec2<f32>(0.22, 0.16));
    let petals = min(left_flower, min(middle_flower, right_flower));

    let left_center = length(point - vec2<f32>(-0.25, 0.11)) - 0.023;
    let middle_center = length(point - vec2<f32>(-0.01, 0.30)) - 0.023;
    let right_center = length(point - vec2<f32>(0.22, 0.16)) - 0.023;
    let centers = min(left_center, min(middle_center, right_center));

    var result = vec4<f32>(0.0);
    result = over(result, vec4<f32>(flower.secondary.rgb, coverage(stems) * flower.secondary.a));
    result = over(result, vec4<f32>(flower.secondary.rgb, coverage(leaves) * flower.secondary.a));
    result = over(result, vec4<f32>(flower.primary.rgb, coverage(petals) * flower.primary.a));
    result = over(result, vec4<f32>(flower.accent.rgb, coverage(centers) * flower.accent.a));
    return result;
}

@fragment
fn fragment(input: FlowerInput) -> @location(0) vec4<f32> {
    let aspect = input.size.x / max(input.size.y, 0.001);
    let point = vec2<f32>((input.uv.x - 0.5) * aspect, 0.5 - input.uv.y);
    let height = clamp(point.y + 0.5, 0.0, 1.0);
    let bent_point = vec2<f32>(point.x - flower.motion.x * height * height, point.y);
    return flower_fragment(bent_point);
}
