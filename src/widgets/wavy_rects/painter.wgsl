struct Settings {
    edge: vec4<f32>,
    fill: vec4<f32>,
    line_width_px: f32,
    corner_radius_px: f32
}

@group(0) @binding(0)
var<uniform> settings: Settings;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    // position get mapped from clip space to viewport (pixel) space between 
    // pipleline stages (looks like)
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.position = vec4<f32>(model.position, 1.0);
    return out;
}

// signed distance from p to a box centered at the origin of size 2*b
fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    var d = abs(p) - b;
    return length(max(d, vec2<f32>())) + min(max(d.x, d.y), 0.0);
}

// p    query point
// b    half box shape (w/2,h/2) where (w,h) is the shape of the box.
// r    corner radius
//
// p,b,r should all be in the same units (metric space). Returns the signed 
// distance between p and the edge of the rounded box. "Inside" corresponds
// to negative distances.
fn sd_round_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, vec2<f32>())) + min(max(q.x, q.y), 0.0) - r;
}

fn premultiply(src: vec4<f32>) -> vec4<f32> {
    return vec4(src.rgb * src.a, src.a);
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    // Scale so distance is evaluated in viewport space.
    // That lets us evaluate the line width in px.
    // Gradient can come out negative depending on triangle orientation,
    // so take the absolute value.
    let duvdx = dpdx(in.tex_coords);  // dvu/dx (tex coord units/viewport pixel)
    let duvdy = dpdy(in.tex_coords);  // duv/dy
    let dx = length(vec2(duvdx.x, duvdy.x));
    let dy = length(vec2(duvdx.y, duvdy.y));
    let s = vec2(dx, dy);

    let d = sd_round_box(in.tex_coords.xy / s, 0.5 / s, settings.corner_radius_px);

    let fill = mix(settings.fill, vec4(0.0), saturate(d));

    if settings.line_width_px < 1e-3 {
        return premultiply(fill);
    } else {
        // attenuate alpha for sub-pixel line-widths
        let edge = vec4(settings.edge.xyz, settings.edge.w * saturate(settings.line_width_px));
        let eps = settings.line_width_px + d; // distance from inner edge of boundary
        return premultiply(mix(mix(edge, fill, saturate(-eps)), vec4(0.0), saturate(d)));
    }
}
