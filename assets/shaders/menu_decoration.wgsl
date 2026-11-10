struct MenuDecorationUniform {
    color: vec4<f32>,
    cursor_position: vec2<f32>,
    shape_id: u32,
    hover: f32,
}

struct MenuDecorationInput {
    @location(0) uv: vec2<f32>,
    @location(2) border_radius_x: vec4<f32>,
    @location(3) border_radius_y: vec4<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
}

const ELLIPSE: u32 = 0u;
const ROUNDED_RECTANGLE: u32 = 1u;

@group(1) @binding(0) var<uniform> decoration: MenuDecorationUniform;

fn ellipse_distance(point: vec2<f32>, radii: vec2<f32>) -> f32 {
    let normalized_length = length(point / radii);
    if normalized_length < 0.0001 {
        return -min(radii.x, radii.y);
    }
    let gradient_length = length(point / (radii * radii));
    return normalized_length * (normalized_length - 1.0) / max(gradient_length, 0.0001);
}

fn rounded_rectangle_distance(
    point: vec2<f32>,
    half_size: vec2<f32>,
    requested_radius: f32,
) -> f32 {
    let radius = clamp(requested_radius, 0.0, min(half_size.x, half_size.y));
    let offset = abs(point) - half_size + vec2<f32>(radius);
    return length(max(offset, vec2<f32>(0.0)))
        + min(max(offset.x, offset.y), 0.0)
        - radius;
}

fn decoration_distance(point: vec2<f32>, input: MenuDecorationInput) -> f32 {
    let half_size = input.size * 0.5;
    switch decoration.shape_id {
        case ELLIPSE: {
            let radii = vec2<f32>(input.border_radius_x.x, input.border_radius_y.x);
            return ellipse_distance(point, radii);
        }
        case ROUNDED_RECTANGLE: {
            return rounded_rectangle_distance(point, half_size, input.border_radius_x.x);
        }
        default: {
            return 1e6;
        }
    }
}

fn decoration_fragment(input: MenuDecorationInput) -> vec4<f32> {
    let point = (input.uv - vec2<f32>(0.5)) * input.size;
    let distance = decoration_distance(point, input);
    let antialias_width = max(fwidth(distance), 0.001);
    let coverage = 1.0 - smoothstep(-0.5 * antialias_width, 0.5 * antialias_width, distance);

    let rim_width = max(2.0 * fwidth(distance), 3.0);
    let rim = (1.0 - smoothstep(0.0, rim_width, max(-distance, 0.0))) * coverage;
    let cursor_distance = length((input.uv - decoration.cursor_position) * input.size);
    let cursor_radius = max(min(input.size.x, input.size.y) * 0.55, 1.0);
    let cursor_glow = exp(-3.2 * cursor_distance * cursor_distance / (cursor_radius * cursor_radius));
    let global_shine = decoration.hover * (0.065 + 0.08 * rim);
    let cursor_shine = decoration.hover * cursor_glow * 0.34;
    let brighten = min(global_shine + cursor_shine, 0.5);
    let color = mix(decoration.color.rgb, vec3<f32>(1.0), brighten);
    let alpha = decoration.color.a * coverage + decoration.hover * rim * 0.1;

    return vec4<f32>(color, min(alpha, 1.0));
}

@fragment
fn fragment(input: MenuDecorationInput) -> @location(0) vec4<f32> {
    return decoration_fragment(input);
}
