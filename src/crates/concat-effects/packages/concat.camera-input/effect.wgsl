struct Params {
    profile: f32,
    levels: f32,
}

// Sony S-Gamut3/S-Log3 Technical Summary and Panasonic V-Log/V-Gamut
// Reference Manual. Matrices convert D65 camera primaries to linear BT.709.
fn sony_log_to_light(v: vec3<f32>) -> vec3<f32> {
    let code = v * 1023.0;
    let toe = (code - 95.0) * (0.01125 / (171.2102946929 - 95.0));
    let log = pow(vec3<f32>(10.0), (code - 420.0) / 261.5) * 0.19 - 0.01;
    return select(toe, log, code >= vec3<f32>(171.2102946929));
}

fn panasonic_log_to_light(v: vec3<f32>) -> vec3<f32> {
    let toe = (v - 0.125) / 5.6;
    let log = pow(vec3<f32>(10.0), (v - 0.598206) / 0.241514) - 0.00873;
    return select(toe, log, v >= vec3<f32>(0.181));
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let source = sample(uv);
    if (params.profile < 0.5) {
        return source;
    }

    // The compositor has unpacked the recorded RGB as display-encoded SDR.
    // Undo that unpacking to recover the code values before camera decoding.
    var code = to_display(source.rgb);
    if (params.levels >= 0.5) {
        code = (code * 876.0 + 64.0) / 1023.0;
    }
    var rgb: vec3<f32>;
    if (params.profile < 1.5) {
        let light = sony_log_to_light(code);
        rgb = vec3<f32>(
            dot(vec3<f32>(1.62694741, -0.54013854, -0.08680887), light),
            dot(vec3<f32>(-0.17851553, 1.41794093, -0.23942540), light),
            dot(vec3<f32>(-0.04443612, -0.19591997, 1.24035608), light),
        );
    } else if (params.profile < 2.5) {
        let light = sony_log_to_light(code);
        rgb = vec3<f32>(
            dot(vec3<f32>(1.87791513, -0.79416876, -0.08374637), light),
            dot(vec3<f32>(-0.17680698, 1.35099962, -0.17419264), light),
            dot(vec3<f32>(-0.02620113, -0.14842226, 1.17462339), light),
        );
    } else {
        let light = panasonic_log_to_light(code);
        rgb = vec3<f32>(
            dot(vec3<f32>(1.806576, -0.695697, -0.110879), light),
            dot(vec3<f32>(-0.170090, 1.305955, -0.135865), light),
            dot(vec3<f32>(-0.025206, -0.154468, 1.179674), light),
        );
    }
    return vec4<f32>(rgb, source.a);
}
