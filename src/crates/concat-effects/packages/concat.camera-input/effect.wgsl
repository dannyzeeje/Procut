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

// Apple Log-to-linear coefficients from the Academy's Apple Log-to-ACES
// transform. Apple Log uses BT.2020 primaries.
fn apple_log_to_light(v: vec3<f32>) -> vec3<f32> {
    let toe = sqrt(max(v, vec3<f32>(0.0)) / 47.28711236) - 0.05641088;
    let log = exp2((v - 0.69336945) / 0.08550479) - 0.00964052;
    return select(toe, log, v >= vec3<f32>(0.20855531595));
}

// ARRI LogC3 SUP 3.x, Linear Scene Exposure Factor, EI 800.
fn arri_logc3_to_light(v: vec3<f32>) -> vec3<f32> {
    let toe = (v - 0.092809) / 5.367655;
    let log = (pow(vec3<f32>(10.0), (v - 0.385537) / 0.247190) - 0.052272) / 5.555556;
    return select(toe, log, v > vec3<f32>(0.149658));
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
    } else if (params.profile < 3.5) {
        let light = panasonic_log_to_light(code);
        rgb = vec3<f32>(
            dot(vec3<f32>(1.806576, -0.695697, -0.110879), light),
            dot(vec3<f32>(-0.170090, 1.305955, -0.135865), light),
            dot(vec3<f32>(-0.025206, -0.154468, 1.179674), light),
        );
    } else if (params.profile < 4.5) {
        let light = apple_log_to_light(code);
        rgb = vec3<f32>(
            dot(vec3<f32>(1.660491002, -0.587641139, -0.072849863), light),
            dot(vec3<f32>(-0.124550475, 1.132899897, -0.008349423), light),
            dot(vec3<f32>(-0.018150763, -0.100578898, 1.118729661), light),
        );
    } else {
        let light = arri_logc3_to_light(code);
        rgb = vec3<f32>(
            dot(vec3<f32>(1.617523436, -0.537286622, -0.080236814), light),
            dot(vec3<f32>(-0.070572741, 1.334613062, -0.264040321), light),
            dot(vec3<f32>(-0.021101728, -0.226953875, 1.248055603), light),
        );
    }
    return vec4<f32>(rgb, source.a);
}
