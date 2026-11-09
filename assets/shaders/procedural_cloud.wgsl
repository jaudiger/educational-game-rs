struct CloudUniform {
    color: vec4<f32>,
    canvas: vec2<f32>,
    shape_id: u32,
    padding: u32,
}

struct MeshCloudInput {
    @location(2) uv: vec2<f32>,
}

struct UiCloudInput {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

const CLOUD_PUFFY: u32 = 0u;
const CLOUD_ELONGATED: u32 = 1u;

@group(2) @binding(0) var<uniform> mesh_cloud: CloudUniform;
@group(1) @binding(0) var<uniform> ui_cloud: CloudUniform;

fn cloud_reference_size(shape_id: u32) -> vec2<f32> {
    if shape_id == CLOUD_ELONGATED {
        return vec2<f32>(220.0, 70.0);
    }
    return vec2<f32>(150.0, 100.0);
}

fn circle_distance(point: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(point - center) - radius;
}

fn capsule_distance(
    point: vec2<f32>,
    start: vec2<f32>,
    end: vec2<f32>,
    radius: f32,
) -> f32 {
    let segment = end - start;
    let along = clamp(dot(point - start, segment) / dot(segment, segment), 0.0, 1.0);
    return length(point - (start + segment * along)) - radius;
}

fn smooth_union(first: f32, second: f32, radius: f32) -> f32 {
    let blend = clamp(0.5 + 0.5 * (second - first) / radius, 0.0, 1.0);
    return mix(second, first, blend) - radius * blend * (1.0 - blend);
}

fn cloud_shape_distance(point: vec2<f32>, shape_id: u32) -> f32 {
    var distance = 1e6;
    switch shape_id {
        case CLOUD_PUFFY: {
            distance = capsule_distance(
                point,
                vec2<f32>(30.0, 63.0),
                vec2<f32>(120.0, 63.0),
                12.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(43.0, 52.0), 22.0),
                5.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(73.0, 41.0), 28.0),
                5.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(106.0, 52.0), 23.0),
                5.0,
            );
        }
        case CLOUD_ELONGATED: {
            distance = capsule_distance(
                point,
                vec2<f32>(22.0, 44.0),
                vec2<f32>(198.0, 44.0),
                13.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(49.0, 38.0), 19.0),
                6.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(88.0, 32.0), 25.0),
                6.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(131.0, 35.0), 24.0),
                6.0,
            );
            distance = smooth_union(
                distance,
                circle_distance(point, vec2<f32>(172.0, 39.0), 19.0),
                6.0,
            );
        }
        default: {}
    }
    return distance;
}

fn cloud_fragment(uv: vec2<f32>, canvas: vec2<f32>, shape_id: u32, color: vec4<f32>) -> vec4<f32> {
    let position = uv * canvas;
    let reference_size = cloud_reference_size(shape_id);
    let point = position * reference_size / canvas;
    let distance = cloud_shape_distance(point, shape_id);
    let aa = max(fwidth(distance), 0.0001);
    let coverage = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, distance);

    let pixel_footprint = max(fwidth(point.x), fwidth(point.y));
    let detail_visibility = 1.0 - smoothstep(1.0, 4.0, pixel_footprint);
    let rim_pixel_width = length(vec2<f32>(dpdx(distance), dpdy(distance)));
    let rim_width = min(max(rim_pixel_width * 1.25, 0.0001), pixel_footprint * 1.5);
    let inner_distance = max(-distance, 0.0);
    let rim = (1.0 - smoothstep(0.0, rim_width, inner_distance)) * detail_visibility;

    var highlight_center = vec2<f32>(50.0, 30.0);
    var highlight_radius = vec2<f32>(29.0, 13.0);
    if shape_id == CLOUD_ELONGATED {
        highlight_center = vec2<f32>(75.0, 18.0);
        highlight_radius = vec2<f32>(43.0, 9.0);
    }
    let highlight_distance = length((point - highlight_center) / highlight_radius);
    let highlight = (1.0 - smoothstep(0.3, 1.0, highlight_distance))
        * detail_visibility;
    let lower_shading = smoothstep(reference_size.y * 0.62, reference_size.y * 0.82, point.y);

    let base_color = vec3<f32>(0.9216, 0.9560, 1.0) * color.rgb;
    let pale_blue = vec3<f32>(0.6654, 0.7835, 0.8879) * color.rgb;
    let rim_color = vec3<f32>(0.4851, 0.6172, 0.7454) * color.rgb;
    var cloud_color = mix(base_color, pale_blue, lower_shading * 0.42);
    cloud_color = mix(cloud_color, rim_color, rim * 0.16);
    cloud_color = mix(cloud_color, vec3<f32>(1.0, 1.0, 1.0) * color.rgb, highlight * 0.28);

    return vec4<f32>(cloud_color, coverage * (230.0 / 255.0) * color.a);
}

@fragment
fn cloud_mesh_fragment(input: MeshCloudInput) -> @location(0) vec4<f32> {
    return cloud_fragment(input.uv, mesh_cloud.canvas, mesh_cloud.shape_id, mesh_cloud.color);
}

@fragment
fn cloud_ui_fragment(input: UiCloudInput) -> @location(0) vec4<f32> {
    return cloud_fragment(input.uv, input.size, ui_cloud.shape_id, ui_cloud.color);
}
