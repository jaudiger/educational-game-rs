struct TreeInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct TreeUniform {
    color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> tree: TreeUniform;

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.0001);
    return 1.0 - smoothstep(-width, width, distance);
}

@fragment
fn fragment(input: TreeInput) -> @location(0) vec4<f32> {
    let x = abs(input.uv.x * 2.0 - 1.0);
    let half_width = input.uv.y;
    let distance = x - half_width;
    let shade = smoothstep(0.05, 0.9, input.uv.y) * 0.035;
    let color = tree.color.rgb * (1.0 - shade);
    return vec4<f32>(color, coverage(distance) * tree.color.a);
}
