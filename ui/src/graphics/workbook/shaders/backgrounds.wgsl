
struct Background {
    bg_types_1: vec4<u32>,
    bg_types_2: vec4<u32>,
    bg_color: vec4<f32>,
}

fn over(front: vec4<f32>, back: vec4<f32>) -> vec4<f32> {
    var ao = front.w + back.w (1-front.w);
    var co = (front.xyz * front.w + back.xyz * back.w * (1 - front.w))/ao;
    return vec4<f32>(co, ao);
}

fn background(bg: Background) -> vec4<f32> {
    var prev = vec4<f32>(0.0,0.0,0.0,0.0);
    for(var i: u32 =0; i<8; i++) {
        switch i {
            case 1: {
                prev = over(bg.bg_color, prev);
            }
            default: {}
        }
    }
    return prev;
}
