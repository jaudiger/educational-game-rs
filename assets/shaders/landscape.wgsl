struct LandscapeInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct LandscapeUniform {
    center_curve: vec4<f32>,
    center_start: vec4<f32>,
    center_end: vec4<f32>,
    path_width: vec4<f32>,
    layer: u32,
    padding: vec3<f32>,
}

const SKY: u32 = 0u;
const MEADOW: u32 = 1u;

@group(1) @binding(0) var<uniform> landscape: LandscapeUniform;

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.001);
    return 1.0 - smoothstep(-width, width, distance);
}

fn path_center_offset(y: f32) -> f32 {
    return landscape.center_curve.x
        + landscape.center_curve.y * smoothstep(landscape.center_start.x, landscape.center_end.x, y)
        + landscape.center_curve.z * smoothstep(landscape.center_start.y, landscape.center_end.y, y)
        + landscape.center_curve.w * smoothstep(landscape.center_start.z, landscape.center_end.z, y);
}

fn path_width(y: f32) -> f32 {
    return mix(
        landscape.path_width.x,
        landscape.path_width.y,
        smoothstep(landscape.path_width.z, landscape.path_width.w, y),
    );
}

fn sky_fragment(input: LandscapeInput) -> vec4<f32> {
    let x = (input.uv.x - 0.5) * input.size.x / max(input.size.y, 0.001);
    let y = input.uv.y;
    let sky = mix(
        vec3<f32>(0.36, 0.67, 0.86),
        vec3<f32>(0.76, 0.87, 0.94),
        smoothstep(0.02, 0.62, y),
    );

    let ridge_center = 0.75;
    let ridge_half_width = select(0.14, 0.13, input.uv.x < ridge_center);
    let ridge_offset = (input.uv.x - ridge_center) / ridge_half_width;
    let ridge_height = 0.14 * sqrt(max(1.0 - ridge_offset * ridge_offset, 0.0));
    let ridge = 0.315 - ridge_height;
    let ridge_strength = smoothstep(0.0, 0.18, 1.0 - abs(ridge_offset));
    let ridge_coverage = coverage(ridge - y) * ridge_strength;
    let color = mix(sky, vec3<f32>(0.48, 0.60, 0.66), ridge_coverage * 0.7);
    return vec4<f32>(color, 1.0);
}

fn meadow_fragment(input: LandscapeInput) -> vec4<f32> {
    let aspect = input.size.x / max(input.size.y, 0.001);
    let x = (input.uv.x - 0.5) * aspect;
    let y = input.uv.y;
    let meadow_line = 0.25 + 0.008 * sin(x * 2.5 + 1.1);
    let meadow_coverage = coverage(meadow_line - y);
    let meadow_shade = smoothstep(0.58, 1.0, y) * 0.32;
    let meadow = mix(
        vec3<f32>(0.54, 0.76, 0.37),
        vec3<f32>(0.42, 0.64, 0.31),
        meadow_shade,
    );

    let path_center = aspect * path_center_offset(y);
    let path_width = path_width(y);
    let path_distance = abs(x - path_center) - path_width;
    let path_coverage = coverage(path_distance) * meadow_coverage;
    let path_edge = vec3<f32>(0.48, 0.37, 0.27);
    var color = mix(meadow, path_edge, path_coverage);

    let edge_width = mix(0.003, 0.012, smoothstep(0.25, 0.98, y));
    let path_center_distance = abs(x - path_center) - (path_width - edge_width);
    let path_center_coverage = coverage(path_center_distance) * meadow_coverage;
    let path_fill = mix(
        vec3<f32>(0.78, 0.66, 0.47),
        vec3<f32>(0.72, 0.56, 0.37),
        smoothstep(0.52, 0.94, y),
    );
    color = mix(color, path_fill, path_center_coverage);
    return vec4<f32>(color, meadow_coverage);
}

@fragment
fn fragment(input: LandscapeInput) -> @location(0) vec4<f32> {
    switch landscape.layer {
        case SKY: {
            return sky_fragment(input);
        }
        case MEADOW: {
            return meadow_fragment(input);
        }
        default: {
            return vec4<f32>(0.0);
        }
    }
}
