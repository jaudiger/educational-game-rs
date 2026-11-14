struct LandscapeInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

struct RidgeGeometry {
    center_x_percent: f32,
    baseline_y: f32,
    left_half_width: f32,
    right_half_width: f32,
    main_height: f32,
    shoulder_height: f32,
    shoulder_offset: f32,
    shoulder_half_width: f32,
}

struct LandscapeUniform {
    center_curve: vec4<f32>,
    center_start: vec4<f32>,
    center_end: vec4<f32>,
    path_width: vec4<f32>,
    ridge: RidgeGeometry,
    layer: u32,
    padding: vec3<f32>,
}

const SKY: u32 = 0u;
const MEADOW: u32 = 1u;
const RIDGE: u32 = 2u;

@group(1) @binding(0) var<uniform> landscape: LandscapeUniform;

fn coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.001);
    return 1.0 - smoothstep(-width, width, distance);
}

fn hill_bump(t: f32) -> f32 {
    let q = max(1.0 - t * t, 0.0);
    return q * q;
}

fn ridge_offset_at(x: f32, aspect: f32, ridge: RidgeGeometry) -> f32 {
    let ridge_center = (ridge.center_x_percent - 0.5) * aspect;
    let ridge_half_width = select(ridge.right_half_width, ridge.left_half_width, x < ridge_center);
    return (x - ridge_center) / ridge_half_width;
}

fn ridge_surface_y(x: f32, aspect: f32, ridge: RidgeGeometry) -> f32 {
    let ridge_center = (ridge.center_x_percent - 0.5) * aspect;
    let ridge_offset = ridge_offset_at(x, aspect, ridge);
    let shoulder_offset =
        (x - ridge_center + ridge.shoulder_offset) / ridge.shoulder_half_width;
    let ridge_height = ridge.main_height * hill_bump(ridge_offset)
        + ridge.shoulder_height * hill_bump(shoulder_offset);
    return ridge.baseline_y - ridge_height;
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

fn sky_color() -> vec3<f32> {
    return vec3<f32>(0.4200333, 0.6920711, 0.9114075);
}

fn sky_fragment(_input: LandscapeInput) -> vec4<f32> {
    return vec4<f32>(sky_color(), 1.0);
}

fn ridge_fragment(input: LandscapeInput) -> vec4<f32> {
    let aspect = input.size.x / max(input.size.y, 0.001);
    let x = (input.uv.x - 0.5) * aspect;
    let y = input.uv.y;
    let ridge_offset = ridge_offset_at(x, aspect, landscape.ridge);
    let ridge_line = ridge_surface_y(x, aspect, landscape.ridge);
    let ridge_coverage = coverage(ridge_line - y);

    let slope_light = smoothstep(-0.7, 0.8, ridge_offset);
    let hill_color = mix(
        vec3<f32>(0.29, 0.48, 0.43),
        vec3<f32>(0.41, 0.59, 0.45),
        slope_light,
    );
    let atmospheric_haze = smoothstep(0.17, 0.25, y) * 0.12;
    let color = mix(hill_color, vec3<f32>(0.48, 0.60, 0.66), atmospheric_haze);
    return vec4<f32>(color, ridge_coverage);
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
        case RIDGE: {
            return ridge_fragment(input);
        }
        default: {
            return vec4<f32>(0.0);
        }
    }
}
