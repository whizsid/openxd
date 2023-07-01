## Rendering Backend

There are lot of rendering backends implemented in rust and some of the backends are targeting
only specific platform. But there are only 2 backends that supports both web and desktop versions.

### Glow

Glow is using the WebGL on browsers and OpenGL on desktop versions. Glow
is easy to integrate with egui on both platforms. And it s a thin library will help us
to reduce the binary size.

### Wgpu

WGPU is same as Glow. But it not limited to OpenGL and WebGL. It using new experimental WebGPU API on
browsers. Also it providing a WebGL emulation which we can use as a fallback method in browser. Also
it will directly supports Vulkan and Metal graphics.

### Decission

One of the biggest advantage of using Glow is that we can reduce the application binary size. Also
it is supporting all the platforms that we need.

But WGPU is more up-to-date and it is directly using the platform specific graphics API. Not like OpenGL.
So we can get a better performance by using WGPU. Also WGPU is supporting GPGPU. Which some of the
browsers not supporting yet. But we can use those features in the future if we used WGPU.

Both graphic backends supporting the egui and we can easily integrate both using the eframe. Therefore
using WGPU will be more ideal.
