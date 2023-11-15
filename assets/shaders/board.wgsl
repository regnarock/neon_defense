struct Parameters {
    screen_x: f32,
    screen_y: f32,
    mouse_x: f32,
    mouse_y: f32,
    is_pressed: f32,
    time: f32,
}

var<push_constant> parameters: Parameters;

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

fn dist_from_hex_center(position: vec2<f32>, CELL_DIMENSIONS: vec2<f32>) -> f32 {
    var p = abs(position);

    var c = dot(p, normalize(CELL_DIMENSIONS));
    c = max(c, p.x);
    return c;
}

// view site: https://www.redblobgames.com/grids/hexagons/
fn cube_subtract(a: FHex, b: FHex) -> vec3<f32> {
    var hex: vec3<f32>;
    hex.x = a.r - b.r;
    hex.y = a.s - b.s;
    hex.z = a.q - b.q;

    return hex;
}

fn cube_distance(a: FHex, b: FHex) -> f32 {
    var r_a = rotate_hex_coord(a);
    var r_b = rotate_hex_coord(b);
    var vec = cube_subtract(r_a, r_b);
    var d = (abs(vec.x) + abs(vec.y) + abs(vec.z)) / 2.;
    d = d / 0.57735; // sqrt(3)/3
    return d;
}

fn in_hex(a: FHex, b: FHex) -> vec3<f32> {
    var r_a = rotate_hex_coord(a);
    var r_b = rotate_hex_coord(b);
    var hex = cube_subtract(r_a, r_b);
    return hex;
}

struct Hex {
    q: i32,
    r: i32,
    s: i32,
}

struct FHex {
    q: f32,
    r: f32,
    s: f32,
}

fn to_fhex(h: Hex) -> FHex {
    var hex: FHex;
    hex.q = f32(h.q);
    hex.r = f32(h.r);
    hex.s = f32(h.s);
    return hex;
}

fn cube_round(frac: FHex) -> Hex {
    var h: Hex;
    h.q = i32(round(frac.q));
    h.r = i32(round(frac.r));
    h.s = i32(round(frac.s));

    var q_diff = abs(f32(h.q) - frac.q);
    var r_diff = abs(f32(h.r) - frac.r);
    var s_diff = abs(f32(h.s) - frac.s);

    if q_diff > r_diff && q_diff > s_diff {
        h.q = -h.r-h.s;
    } else if r_diff > s_diff {
        h.r = -h.q-h.s;
    } else {
        h.s = -h.q-h.r;
    }
    return h;
}

fn pixel_to_hex(point: vec2<f32>, size: f32) -> Hex {
    return cube_round(pixel_to_fhex(point, size));
}

fn fhex_to_pixel(fhex: FHex, size: f32) -> vec2<f32> {
    var point: vec2<f32>;
    // vertical/pointy design
    point.x = size * (sqrt(3.) * fhex.q  +  (sqrt(3.)/2. * fhex.r));
    point.y = size * (                      (3./2. * fhex.r));
    // horizontal/falt design
    //point.x = size * (3./2. * fhex.q);
    //point.y = size * (sqrt(3.)/2. * fhex.q  +  sqrt(3.) * fhex.r);
    return point;
}

fn rotate_hex_coord(fhex: FHex) -> FHex {
    var hex: FHex;
    hex.q = (2. * fhex.q + fhex.r) * sqrt(3.) / 3.;
    hex.r = (fhex.r - fhex.q) * sqrt(3.) / 3.;
    hex.s = -hex.q - hex.r;
    return hex;
}

fn pixel_to_fhex(point: vec2<f32>, size: f32) -> FHex {
    var hex: FHex;
    // vertical/pointy design
    hex.q = (sqrt(3.)/3. * point.x  -  1./3. * point.y) / size;
    hex.r = (                          2./3. * point.y) / size;
    // horizontal/falt design
    //hex.q = ( 2./3. * point.x                        ) / size;
    //hex.r = (-1./3. * point.x + sqrt(3.)/3. * point.y) / size;
    hex.s = -hex.q - hex.r;
    return hex;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    var mouse: vec2<f32> = vec2<f32>(parameters.mouse_x, parameters.mouse_y);
    mouse.x -= parameters.screen_x / 2.;
    mouse.y -= parameters.screen_y / 2.;
    mouse /= vec2<f32>(parameters.screen_x, parameters.screen_y);
    let SCALE = .05;
    let CELL_DIMENSIONS = vec2<f32>(3./2., 1.7320508076) * SCALE;

    var tmp_invocation_id = vec2<f32>(invocation_id.xy) - vec2<f32>(parameters.screen_x, parameters.screen_y) / 2.;
    var uv = vec2<f32>(tmp_invocation_id.xy) / vec2<f32>(parameters.screen_x, parameters.screen_y);

    // uv *= 10.;
    var hex = pixel_to_hex(uv, SCALE);
    var hex_m = pixel_to_hex(vec2<f32>(mouse.x, mouse.y), SCALE);
    var c_i = 0.08;
    var color: vec3<f32>;
    //color = vec3<f32>(0.867, 0.145, 0.145);
    color = vec3<f32>(f32(hex.q) * c_i, f32(hex.r) * c_i, f32(hex.s) * c_i);

    var cf = pixel_to_fhex(uv, SCALE);
    var d = cube_distance(cf, to_fhex(hex));
    // outline
    if hex_m.q == hex.q && hex_m.r == hex.r && hex_m.s == hex.s {
        var centerDist = ((d - d % 0.2) - parameters.time * 2.) % 1.;
        var loweredRed = vec3<f32>(centerDist, 0.0, 0.0);
        color = mix(color, loweredRed, centerDist);
        //color = mix(color, vec3<f32>(1.0, .0, .0), step(0.8, d));
        //color = vec3<f32>(cf.q, cf.r, cf.s);
    } else {
    //     let aa = 10. / parameters.screen_y;
    //     let external_intensity = (smoothstep(.51, .51 - aa, d) + pow(1. - max(0., .5 - d), 20.) * 1.5);
    //     //color *= external_intensity;
    }
    if d > 0.97 {
        color = mix(color, vec3<f32>(1., 0.0, 0.0), smoothstep(0.97, 1., d));
    }

    //1 - smoothstep(0.0, 3.0, (abs(d) - 0.002) * iResolution.y)
    textureStore(texture, invocation_id.xy, vec4<f32>(color, 1.0));
}
