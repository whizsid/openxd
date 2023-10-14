struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) bbox: vec4<f32>,
    @location(2) bg_types_1: vec4<u32>,
    @location(3) bg_types_2: vec4<u32>,
    @location(4) bg_color: vec4<f32>
};

struct Rectangle {
    @location(0) tl: vec2<f32>,
    @location(1) tr: vec2<f32>,
    @location(2) bl: vec2<f32>,
    @location(3) br: vec2<f32>,
    @location(4) bbox: vec4<f32>,
    @location(5) bg_types_1: vec4<u32>,
    @location(6) bg_types_2: vec4<u32>,
    @location(7) bg_color: vec4<f32>,
    @location(8) depth: u32
}

struct Background {
    bg_types_1: vec4<u32>,
    bg_types_2: vec4<u32>,
    bg_color: vec4<f32>,
}

fn over(front: vec4<f32>, back: vec4<f32>) -> vec4<f32> {
    var ao = front.w + back.w * (1.0-front.w);
    var co = (front.xyz * front.w + back.xyz * back.w * (1.0 - front.w))/ao;
    return vec4<f32>(co, ao);
}

fn background(bg: Background) -> vec4<f32> {
    var prev = vec4<f32>(0.0,0.0,0.0,0.0);
    for(var i: i32 =0; i<8; i++) {
        var bg_type: u32;
        var bg_types: vec4<u32> = bg.bg_types_1;
        var j = u32(i);
        if j>3u {
            j = j - 4u;
            bg_types = bg.bg_types_2;
        }
        switch j {
            case 0u: {
                j = bg_types.x;
            }
            case 1u: {
                j = bg_types.y;
            }
            case 2u: {
                j = bg_types.z;
            }
            case 3u: {
                j = bg_types.w;
            }
            default: {}
        }
        switch j {
            case 1u: {
                prev = over(bg.bg_color, prev);
            }
            default: {}
        }
    }
    return prev;
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    rect: Rectangle 
) -> VertexOutput {
    var out: VertexOutput;

    out.color = vec4<f32>(1.0,0.0,1.0,1.0);
    out.bbox = rect.bbox;
    out.bg_types_1 = rect.bg_types_1;
    out.bg_types_2 = rect.bg_types_2;
    out.bg_color = rect.bg_color;

    switch in_vertex_index {
        case 0u: {
            out.position = vec4<f32>(rect.tl, 0.0, 1.0);
        }
        case 1u: {
            out.position = vec4<f32>(rect.tr, 0.0, 1.0);
        }
        case 2u: {
            out.position = vec4<f32>(rect.bl, 0.0, 1.0);
        }
        case 3u: {
            out.position = vec4<f32>(rect.br, 0.0, 1.0);
        }
        case 4u: {
            out.position = vec4<f32>(rect.tr, 0.0, 1.0);
        }
        case 5u: {
            out.position = vec4<f32>(rect.bl, 0.0, 1.0);
        }
        default: {
        }
    }
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var bg: Background;
    bg.bg_types_1 = in.bg_types_1;
    bg.bg_types_2 = in.bg_types_2;
    bg.bg_color = in.bg_color;
    return background(bg);
}
