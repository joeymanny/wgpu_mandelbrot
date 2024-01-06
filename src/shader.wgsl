@group(0) @binding(0) var<uniform> cmd: MandelCommands;

struct MandelCommands{
    size: vec2<f32>,
    offset: vec2<f32>,
    zoom: f32,
    lod: f32
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @builtin(instance_index) instance_index: u32) -> @builtin(position) vec4<f32> {
    // let x = f32(i32(in_vertex_index) - 1);
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    var x: f32 = 0.0;
    var y: f32 = 0.0;
    // square
    if in_vertex_index == u32(0){
        x = -1.;
        y = -1.;
    } else if in_vertex_index == u32(1){
        if instance_index == u32(0){
            x = -1.;
            y = 1.;
        }else{
            x = 1.;
            y = -1.;
        }
    }else if in_vertex_index == u32(2){
        x = 1.;
        y = 1.;
    }

    // triangle
    // if in_vertex_index == u32(0){
    //     x = -1.;
    //     y = -1.;
    // }else if in_vertex_index == u32(1){
    //     x = 0.;
    //     y = 1.
    // }else{
    //     x = 1.;
    //     y = -1.;
    // }
    return vec4<f32>(f32(x), f32(y), 0.0, 1.0);
}


@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // let c1 = vec3(0.988, 0.196, 0.31);
    // let c2 = vec3(0.137, 0.659, 0.949);
    // let c1 = vec3(1., 0., 0.,);
    // let c2 = vec3(0., 0., 1.);
    
    var newpos = pos.xy / cmd.size;
    newpos.x = ((newpos.x - 0.5) * (cmd.size.x / cmd.size.y)) * pow(2.0, cmd.zoom);
    newpos.y = (newpos.y - 0.5) * pow(2.0, cmd.zoom);
    newpos += cmd.offset;

    let mand = f32(mandel(Complex(newpos.x, newpos.y))) / cmd.lod;
    // var circle = vec3(.5, .5, .3);

    // var dist = length(newpos - circle.xy) - circle.z;

    // dist = smoothstep(0.0, 0.005, dist);

    let col = mix(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0), mand - .1);
    
    return vec4<f32>(col, 1.0);

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
    return Complex(pow(in.real, 2.0) - pow(in.imag, 2.0), f32(2.0) * in.real * in.imag);
}

fn abs_sq(in: Complex) -> f32 {
    return (pow(in.real, 2.0) + pow(in.imag, 2.0));
}

fn mandel(c: Complex) -> u32 {
    var z = Complex(0.0, 0.0);
    var i: u32 = u32(0);
    while (i < u32(cmd.lod)){
        i += u32(1);
        z = mandel_iter(z, c);
        if abs_sq(z) > 4.0 {
            return i;
        }
    }
    return i;
}