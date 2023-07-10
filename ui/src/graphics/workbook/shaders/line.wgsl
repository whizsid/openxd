// Vertex shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) width: f32,
    @location(2) stroke: u32,
    @location(3) start: vec2<f32>,
    @location(4) end: vec2<f32>
};

struct Line {
    @location(0) tl: vec2<f32>,
    @location(1) tr: vec2<f32>,
    @location(2) bl: vec2<f32>,
    @location(3) br: vec2<f32>,
    @location(4) depth: u32,
    @location(5) color: vec4<f32>,
    @location(6) width: f32,
    @location(7) stroke: u32,
    @location(8) start: vec2<f32>,
    @location(9) end: vec2<f32>,
}

fn marker(center: vec2<f32>, position: vec2<f32>) -> bool {
    if(distance(center, position) < 20.0) {
        return true;
    } else {
        return false;
    }
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    line: Line
) -> VertexOutput {
    var out: VertexOutput;
    out.color = line.color;
    out.width = line.width;
    out.stroke = line.stroke;
    out.start = line.start;
    out.end = line.end;


    switch in_vertex_index {
        case 0u: {
            out.position = vec4<f32>(line.tl, 0.0, 1.0);
        }
        case 1u: {
            out.position = vec4<f32>(line.tr, 0.0, 1.0);
        }
        case 2u: {
            out.position = vec4<f32>(line.bl, 0.0, 1.0);
        }
        case 3u: {
            out.position = vec4<f32>(line.br, 0.0, 1.0);
        }
        case 4u: {
            out.position = vec4<f32>(line.tr, 0.0, 1.0);
        }
        case 5u: {
            out.position = vec4<f32>(line.bl, 0.0, 1.0);
        }
        default: {
        }
    }
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    switch in.stroke {
        case 1u: {
            var a = in.start.x;
            var b = in.start.y;
            var c = in.end.x;
            var d = in.end.y;

            var l = in.position.x;
            var m = in.position.y;

            var h = abs((d-b)*l - (c-a)*m -a*(d-b) + b*(c-a))/sqrt(pow(d-b,2.0) + pow(c-a,2.0));

            if(h<in.width*0.2) {
                discard;
            }

            return in.color;
        }
        default: {
            return in.color;
        }
    }


    return in.color;
}
