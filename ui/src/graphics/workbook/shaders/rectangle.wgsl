struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(5) bbox: vec4<f32>
};

struct Rectangle {
    @location(0) tl: vec2<f32>,
    @location(1) tr: vec2<f32>,
    @location(2) bl: vec2<f32>,
    @location(3) br: vec2<f32>,
    @location(10) bbox: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    rect: Rectangle 
) -> VertexOutput {
    var out: VertexOutput;

    out.color = vec4<f32>(1.0,0.0,1.0,1.0);
    out.bbox = rect.bbox;

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
    return in.color;
}
