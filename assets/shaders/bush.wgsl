struct BushInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct BushUniform {
    primary: vec4<f32>,
    highlight: vec4<f32>,
}

@group(1) @binding(0) var<uniform> bush: BushUniform;

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.0001);
    return 1.0 - smoothstep(-width, width, distance);
}

fn ellipse_field(point: vec2<f32>, center: vec2<f32>, radii: vec2<f32>) -> f32 {
    let normalized = (point - center) / radii;
    return dot(normalized, normalized) - 1.0;
}

fn shrub_field(point: vec2<f32>) -> f32 {
    let left = ellipse_field(point, vec2<f32>(-0.48, 0.28), vec2<f32>(0.52, 0.48));
    let center = ellipse_field(point, vec2<f32>(0.0, 0.42), vec2<f32>(0.60, 0.58));
    let right = ellipse_field(point, vec2<f32>(0.48, 0.28), vec2<f32>(0.52, 0.48));
    return min(left, min(center, right));
}

@fragment
fn fragment(input: BushInput) -> @location(0) vec4<f32> {
    let point = vec2<f32>((input.uv.x - 0.5) * 2.0, 1.0 - input.uv.y);
    let shade = smoothstep(0.0, 1.0, point.y) * 0.28;
    let color = mix(bush.primary.rgb, bush.highlight.rgb, shade);
    return vec4<f32>(color, coverage(shrub_field(point)) * bush.primary.a);
}
