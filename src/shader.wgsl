// @vertex
// fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
//     let x = f32(i32(in_vertex_index) - 1);
//     let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
//     return vec4<f32>(x, y, 0.0, 1.0);
// }

// @fragment
// fn fs_main(@builtin(position) coord_in: vec4<f32>) -> @location(0) vec4<f32> {

//     return vec4<f32>(1.0, 0.5, 0.0, 1.0);
//     // return coord_in * f32(0.5);

// }

@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) coord_in: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4<f32>(coord_in.x, coord_in.y, 0.0, 1.0);
}

struct Complex{
    real: f32,
    imag: f32,
}

fn mandel_iter(in: Complex, c: Complex) -> Complex {
    var ret = sqr(in);
    ret.real += c.real;
    ret.imag += c.imag;
    return ret;
}

fn sqr(in: Complex) -> Complex {
    return Complex(f32(pow(in.real, 2.0) - pow(in.imag, 2.0)), f32(f32(2.0) * in.real * in.imag));
}

fn abs_sq(in: Complex) -> f32 {
    return (pow(in.real, 2.0) + pow(in.imag, 2.0));
}

fn mandel(c: Complex) -> u32 {
    var z = Complex(0.0, 0.0);
    var i: u32 = u32(0);
    while (i < u32(2000) ){
        i += u32(1);
        z = mandel_iter(z, c);
        if abs_sq(z) > 4.0 {
            break;
        }
    }
    return i;
}