var<private> v_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
);

struct VertexOut {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32
) -> VertexOut {
    var out: VertexOut;
    let vert = v_positions[v_idx];

    out.clip_position = vec4<f32>(vert, 0.0, 1.0);
    out.tex_coords = vec2<f32>(
        (vert.x + 1.0) / 2.0,
        (-vert.y + 1.0) / 2.0,
    );

    return out;
}

@group(0) @binding(0)
var<uniform> screen: vec2<f32>;
@group(0) @binding(1)
var<uniform> rect: vec4<f32>;
@group(0) @binding(2)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(3)
var s_diffuse: sampler;

var<private> pi: f32 = 3.141592653589793;

// vec4 color = vec4(0.0);
//   vec2 off1 = vec2(1.3846153846) * direction;
//   vec2 off2 = vec2(3.2307692308) * direction;
//   color += texture2D(image, uv) * 0.2270270270;
//   color += texture2D(image, uv + (off1 / resolution)) * 0.3162162162;
//   color += texture2D(image, uv - (off1 / resolution)) * 0.3162162162;
//   color += texture2D(image, uv + (off2 / resolution)) * 0.0702702703;
//   color += texture2D(image, uv - (off2 / resolution)) * 0.0702702703;
//   return color;
// }

// fn blur(

// )

@fragment
fn fs_main(
    in: VertexOut
) -> @location(0) vec4<f32> {
    let x = rect.x;
    let y = rect.y;
    let width = rect.z - x;
    let height = rect.w - y;

    let coord_x = x + (width  * in.tex_coords.x);
    let coord_y = y + (height * in.tex_coords.y);

    let sigma = 2.0;
    let k = 2.0 * sigma * sigma;

    let size = i32(floor(sigma * 3.0));

    // let hsize = f32(2 * size + 1);
    // let fac = 1.0 / (hsize * hsize);

    var rgba = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    for(var i: i32 = -size; i <= size; i++) {
        for(var j: i32 = -size; j <= size; j++) {
            // rgba += vec4<f32>(fac, fac, fac, fac);
            // rgba *= 1.0;
            let i_f32 = f32(i);
            let j_f32 = f32(j);

            let fac = exp(-(i_f32*i_f32 + j_f32*j_f32) / k) / (pi * k);
            // let fac = 0.0;

            let pos = vec2<f32>(
                (coord_x + f32(i)) / screen.x,
                (coord_y + f32(j)) / screen.y
            );

            let sampled = textureSample(
                t_diffuse, s_diffuse,
                pos
            );

            rgba += vec4<f32>(
                sampled.x * fac,
                sampled.y * fac,
                sampled.z * fac,
                0.0
            );

            // rgba += (textureSample(
            //     t_diffuse, s_diffuse,
            //     (coord_x + f32(i)) / screen.x,
            //     (coord_y + f32(j)) / screen.y,
            // ) * fac);
        }
    }

    rgba = mix(
        rgba,
        vec4<f32>(0.0, 0.5, 0.6, 1.0),
        // pow(0.01, 1.22)
        0.0
    );

    return rgba;
}