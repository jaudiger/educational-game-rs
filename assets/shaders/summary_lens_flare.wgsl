struct SummaryLensFlareUniform {
    source: vec4<f32>,
    cursor: vec4<f32>,
}

struct SummaryLensFlareInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

@group(1) @binding(0) var<uniform> flare: SummaryLensFlareUniform;

fn lens_ghost(
    point: vec2<f32>,
    center: vec2<f32>,
    radius: f32,
    color: vec3<f32>,
    intensity: f32,
) -> vec4<f32> {
    let distance = length(point - center);
    let edge_width = max(fwidth(distance), 0.75);
    let ring_distance = abs(distance - radius * 0.68);
    let ring_width = radius * 0.075;
    let ring = 1.0 - smoothstep(ring_width, ring_width + edge_width, ring_distance);
    let core = exp(-distance * distance / max(radius * radius * 0.09, 1.0));
    let haze = exp(-distance * distance / max(radius * radius * 0.8, 1.0));
    let alpha = clamp((ring * 0.36 + core * 0.13 + haze * 0.045) * intensity, 0.0, 0.42);
    return vec4<f32>(color, alpha);
}

fn over(top: vec4<f32>, bottom: vec4<f32>) -> vec4<f32> {
    let alpha = top.a + bottom.a * (1.0 - top.a);
    let color = (
        top.rgb * top.a + bottom.rgb * bottom.a * (1.0 - top.a)
    ) / max(alpha, 0.0001);
    return vec4<f32>(color, alpha);
}

@fragment
fn fragment(input: SummaryLensFlareInput) -> @location(0) vec4<f32> {
    let source_radius = flare.source.z;
    if source_radius <= 0.0 {
        return vec4<f32>(0.0);
    }

    let point = input.uv * input.size;
    let source = flare.source.xy * input.size;
    let cursor = flare.cursor.xy * input.size;
    let to_cursor = cursor - source;
    let cursor_distance = length(to_cursor);
    let axis = to_cursor / max(cursor_distance, 1.0);
    let extension = min(cursor_distance * 0.16, source_radius * 0.8);
    let first_distance = source_radius + extension * 0.25;
    let second_distance = source_radius * 1.55 + extension * 0.6;
    let third_distance = source_radius * 2.1 + extension;
    let first_center = source + axis * first_distance;
    let second_center = source + axis * second_distance;
    let third_center = source + axis * third_distance;

    var ghosts = vec4<f32>(0.0);
    ghosts = over(
        lens_ghost(
            point,
            first_center,
            source_radius * 0.28,
            vec3<f32>(0.45, 0.82, 1.0),
            0.8 * flare.source.w,
        ),
        ghosts,
    );
    ghosts = over(
        lens_ghost(
            point,
            second_center,
            source_radius * 0.42,
            vec3<f32>(1.0, 0.42, 0.72),
            0.72 * flare.source.w,
        ),
        ghosts,
    );
    ghosts = over(
        lens_ghost(
            point,
            third_center,
            source_radius * 0.2,
            vec3<f32>(0.68, 1.0, 0.5),
            0.85 * flare.source.w,
        ),
        ghosts,
    );

    if cursor_distance <= 1.0 {
        return vec4<f32>(0.0);
    }
    return ghosts;
}
