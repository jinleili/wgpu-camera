
// https://software.intel.com/en-us/blogs/2014/07/15/an-investigation-of-fast-real-time-gpu-based-image-blur-algorithms

struct InfoParams {
  img_width: i32;
  img_height: i32;
};

[[group(0), binding(0)]] var<uniform> params: InfoParams;
[[group(0), binding(1)]] var blur_xy: texture_storage_2d<rgba8unorm, write>;
[[group(0), binding(2)]] var swap_x: texture_2d<f32>;

let RADIUS: i32 = 4;
let WEIGHT: array<f32, 5> = array<f32, 5>(0.12, 0.11, 0.11, 0.11, 0.11);
let UV_ZERO: vec2<i32> = vec2<i32>(0, 0);

[[stage(compute), workgroup_size(16, 16)]]
fn cs_main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let uv: vec2<i32> = vec2<i32>(global_invocation_id.xy);
  if (uv.x >= params.img_width || uv.y >= params.img_height) {
    return;
  }
   // 在 metal(macOS, iOS)上，上面的共享缓存方案性能更差
  var temp: vec4<f32> = textureLoad(swap_x, uv, 0) * WEIGHT[0];
  var uvMax: vec2<i32> = vec2<i32>(params.img_width - 1, params.img_height - 1);
  for (var i: i32 = 1; i <= RADIUS; i += 1) {
    var uvOffset: vec2<i32> = vec2<i32>(0, 3) * i;
    temp += textureLoad(swap_x, clamp(uv + uvOffset, UV_ZERO, uvMax), 0) * 0.11;
    temp += textureLoad(swap_x, clamp(uv - uvOffset, UV_ZERO, uvMax), 0) * 0.11;
  }
  textureStore(blur_xy, uv, temp);
}
