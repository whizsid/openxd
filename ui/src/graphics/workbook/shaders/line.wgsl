// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Line {
    @location(0) tl: vec2<f32>,
    @location(1) tr: vec2<f32>,
    @location(2) bl: vec2<f32>,
    @location(3) br: vec2<f32>,
    @location(4) depth: u32,
    @location(5) color: vec4<f32>
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    line: Line
) -> VertexOutput {
    var out: VertexOutput;
    out.color = line.color;
    switch in_vertex_index {
        case 0u: {
            out.clip_position = vec4<f32>(line.tl, 0.0, 1.0);
        }
        case 1u: {
            out.clip_position = vec4<f32>(line.tr, 0.0, 1.0);
        }
        case 2u: {
            out.clip_position = vec4<f32>(line.bl, 0.0, 1.0);
        }
        case 3u: {
            out.clip_position = vec4<f32>(line.br, 0.0, 1.0);
        }
        case 4u: {
            out.clip_position = vec4<f32>(line.tr, 0.0, 1.0);
        }
        case 5u: {
            out.clip_position = vec4<f32>(line.bl, 0.0, 1.0);
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
