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

const DOUBLE_GAP:f32 = 0.1;
const DASHED_GAP:f32 = 2.0;
const DASHED_FILL:f32 = 2.0;
const LONG_DASHED_GAP:f32 = 2.0;
const LONG_DASHED_FILL:f32 = 4.0;
const DIAMOND_GAP:f32 = 0.5;
const DOT_GAP:f32 = 0.5;

fn distance_line_edges(start: vec2<f32>, end: vec2<f32>, pos: vec2<f32>) -> vec2<f32> {
    let a = start.x;
    let b = start.y;
    let c = end.x;
    let d = end.y;

    let i = pos.x;
    let j = pos.y;

    // Closest distance point on the line (u, v)
    // ```
    // 
    //     -(c-a)[-(c-a)i - (d-b)j] - (d-b)[-a(d-b) + b(c-a)]
    // u = --------------------------------------------------
    //                  (d-b)^2 + (c-a)^2
    //
    //     (d-b)[(c-a)i + (d-b)j] + (c-a)[-a(d-b) + b(c-a)]
    // v = -------------------------------------------------
    //                  (d-b)^2 + (c-a)^2
    // ```

    let u = ((c - a) * ((c - a) * i + (d - b) * j) + (d - b) * (a * (d - b) - b * (c - a))) / (pow(d - b, 2.) + pow(c - a, 2.));
    let v = ((d - b) * ((c - a) * i + (d - b) * j) + (c - a) * (b * (c - a) - a * (d - b))) / (pow(d - b, 2.) + pow(c - a, 2.));

    let closest_point = vec2<f32>(u, v);

    return vec2<f32>(distance(start, closest_point), distance(end, closest_point));
}

fn distance_line(start: vec2<f32>, end: vec2<f32>, pos: vec2<f32>) -> f32 {
    let a = start.x;
    let b = start.y;
    let c = end.x;
    let d = end.y;

    let i = pos.x;
    let j = pos.y;
    // Equation of the line
    // ```
    // (d-b)x - (c-a)y - a(d-b) + b(c-a) = 0
    // ```
    //
    // Distance to the line from (i,j)
    // ```
    //     |(d-b)i - (c-a)j - a(d-b) + b(c-a)|
    // h = -----------------------------------
    //           √((d-b)^2 + (c-a)^2)
    // ```

    let h = abs((d - b) * i - (c - a) * j - a * (d - b) + b * (c - a)) / sqrt(pow(d - b, 2.0) + pow(c - a, 2.0));

    return h;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    switch in.stroke {
        case 1u: {
            let h = distance_line(in.start, in.end, in.position.xy);

            if h < in.width * DOUBLE_GAP {
                discard;
            }

            return in.color;
        }
        case 2u: {
            let dis = distance_line_edges(in.start, in.end, in.position.xy);

            if dis.y < in.width * DASHED_GAP {
                return in.color;
            }

            let pattern = in.width * (DASHED_GAP + DASHED_FILL);
            let remaining = dis.x % pattern;
            if remaining < in.width * DASHED_FILL {
                return in.color;
            }
            discard;
        }
        case 3u: {
            let dis = distance_line_edges(in.start, in.end, in.position.xy);

            if dis.y < in.width * LONG_DASHED_GAP {
                return in.color;
            }

            let pattern = in.width * (LONG_DASHED_GAP + LONG_DASHED_FILL);
            let remaining = dis.x % pattern;
            if remaining < in.width * LONG_DASHED_FILL {
                return in.color;
            }
            discard;
        }
        case 4u: {
            let h = distance_line(in.start, in.end, in.position.xy);
            let dis = distance_line_edges(in.start, in.end, in.position.xy);

            let pattern = in.width * (1.0 + DIAMOND_GAP);
            var l = dis.x % pattern;

            if l > in.width {
                discard;
            }

            if l > in.width * 0.5 {
                l = in.width - l;
            }

            if h / l > 1.0 {
                discard;
            }

            return in.color;
        }
        case 5u: {
            let h = distance_line(in.start, in.end, in.position.xy);
            let dis = distance_line_edges(in.start, in.end, in.position.xy);

            let pattern = in.width * (1.0 + DIAMOND_GAP);
            var l = dis.x % pattern;

            if l > in.width {
                discard;
            }

            if l > in.width * 0.5 {
                l = l - in.width * 0.5;
            } else {
                l = in.width * 0.5 - l;
            }

            let rad = sqrt(pow(h, 2.0) + pow(l, 2.0));

            if rad > in.width*0.5 {
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
