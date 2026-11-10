struct Globals {
    time: f32,
    delta_time: f32,
    frame_count: u32,
}

struct SummarySunUniform {
    glare: vec4<f32>,
    geometry: vec4<f32>,
}

struct SummarySunInput {
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(1) var<uniform> globals: Globals;
@group(1) @binding(0) var<uniform> sun: SummarySunUniform;

const TAU: f32 = 6.28318530718;

fn rounded_box_distance(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let offset = abs(point) - half_size + vec2<f32>(radius);
    return length(max(offset, vec2<f32>(0.0)))
        + min(max(offset.x, offset.y), 0.0)
        - radius;
}

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.001);
    return 1.0 - smoothstep(-0.5 * width, 0.5 * width, distance);
}

fn rotate(point: vec2<f32>, angle: f32) -> vec2<f32> {
    let sine = sin(angle);
    let cosine = cos(angle);
    return vec2<f32>(
        cosine * point.x - sine * point.y,
        sine * point.x + cosine * point.y,
    );
}

fn sparkle_coverage(
    point: vec2<f32>,
    center: vec2<f32>,
    size: f32,
    time: f32,
    phase: f32,
) -> f32 {
    let twinkle = sin(time * 2.6 + phase);
    let half_length = size * (0.9 + twinkle * 0.25);
    let half_width = size * 0.16;
    let local = rotate(point - center, sin(time * 0.45 + phase) * 0.12);
    let vertical = rounded_box_distance(
        local,
        vec2<f32>(half_width, half_length),
        half_width,
    );
    let horizontal = rounded_box_distance(
        local,
        vec2<f32>(half_length, half_width),
        half_width,
    );
    return max(coverage(vertical), coverage(horizontal)) * (0.8 + max(twinkle, 0.0) * 0.2);
}

@fragment
fn fragment(input: SummarySunInput) -> @location(0) vec4<f32> {
    let time = globals.time;
    let pulse = 1.0 + sin(time * 0.8) * 0.01;
    let point = (input.uv - vec2<f32>(0.5)) * 2.0 * sun.geometry.x / pulse;
    let radius = length(point);
    let ray_rotation = sin(time * 0.7) * 0.07;

    var ray_distance = 1e6;
    for (var index = 0u; index < 8u; index += 1u) {
        let phase = f32(index) * TAU / 8.0 + ray_rotation;
        let half_length = select(0.22, 0.08, index % 2u == 1u);
        let center_radius = 0.70 + half_length;
        let local = rotate(point, -phase) - vec2<f32>(0.0, -center_radius);
        let distance = rounded_box_distance(
            local,
            vec2<f32>(0.038, half_length),
            0.038,
        );
        ray_distance = min(ray_distance, distance);
    }
    let ray_alpha = coverage(ray_distance) * 0.94;
    let outer_disc = coverage(radius - 0.78) * 0.46;
    let middle_disc = coverage(radius - 0.655);
    let inner_disc = coverage(radius - 0.511);
    let glow = exp(-max(radius - 0.42, 0.0) * 7.0)
        * (1.0 - smoothstep(0.78, 1.12, radius))
        * 0.2;

    var color = vec3<f32>(1.0, 0.47, 0.08);
    color = mix(color, vec3<f32>(1.0, 0.55, 0.12), outer_disc);
    color = mix(color, vec3<f32>(1.0, 0.62, 0.17), middle_disc);
    color = mix(color, vec3<f32>(1.0, 0.82, 0.3), inner_disc);

    let glare_direction = sun.glare.xy;
    let glare_tangent = vec2<f32>(-glare_direction.y, glare_direction.x);
    let glare_offset = point - glare_direction * 0.22;
    let glare_point = vec2<f32>(
        dot(glare_offset, glare_tangent) * 4.5,
        dot(glare_offset, glare_direction) * 7.0,
    );
    let glare_strength = clamp(sun.glare.z, 0.0, 1.0);
    let highlight = exp(-dot(glare_point, glare_point))
        * 0.78 * glare_strength * inner_disc;
    let broad_sheen_point = glare_point * vec2<f32>(0.42, 0.55);
    let broad_sheen = exp(-dot(broad_sheen_point, broad_sheen_point))
        * 0.22 * glare_strength * inner_disc;
    let radial_direction = point / max(radius, 0.0001);
    let rim_alignment = pow(max(dot(radial_direction, glare_direction), 0.0), 12.0);
    let rim_distance = (radius - 0.73) / 0.075;
    let rim_glare = exp(-rim_distance * rim_distance) * rim_alignment * 0.52 * glare_strength;
    color = mix(color, vec3<f32>(1.0, 0.96, 0.73), highlight);
    color = mix(color, vec3<f32>(1.0, 0.98, 0.82), broad_sheen);
    color = mix(color, vec3<f32>(1.0, 0.84, 0.56), rim_glare);
    color = mix(color, vec3<f32>(1.0, 0.62, 0.17), ray_alpha);

    var alpha = max(glow, max(ray_alpha, outer_disc));
    alpha = max(alpha, max(middle_disc, inner_disc));

    let sparkle_alpha = max(
        sparkle_coverage(point, vec2<f32>(-0.588, -0.348), 0.112, time, 0.0),
        max(
            sparkle_coverage(point, vec2<f32>(0.586, -0.394), 0.086, time, 1.4),
            sparkle_coverage(point, vec2<f32>(0.552, 0.472), 0.112, time, 4.2),
        ),
    );
    color = mix(color, vec3<f32>(1.0, 0.98, 0.82), sparkle_alpha);
    alpha = max(alpha, sparkle_alpha);

    return vec4<f32>(color, alpha);
}
