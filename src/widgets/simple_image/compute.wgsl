struct Settings {
    time: f32
}

@group(0)
@binding(0)
var out: texture_storage_2d<rgba8unorm, write>;

@group(0)
@binding(1)
var<uniform> settings:Settings;

@compute 
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let r = vec2<i32>(global_id.xy);
    let c = vec4(vec2<f32>(r) * 1.0e-2, 1.0, 1.0);
    textureStore(out, r, c);
}