# UI framework

We should adopt a UI framework that supporting web and desktop. Also we have a requirement
that the UI framework should support to control the GL using a platform
agnostic way([glow](https://github.com/grovesNL/glow)). There are several
UI frameworks meets those requirements. But have some disadvantages and
advantages.

## Choices

### iced-rs

[iced-rs](https://github.com/iced-rs/iced) is a popular rust UI kit that supporting both platforms.
Also it supporting to control GL using the `glow`. Also it adopted the Elm design pattern which ideal
for large projects. But it still not supporting [Multi-window support](https://github.com/iced-rs/iced/issues/27),
[Menu bars support](https://github.com/iced-rs/iced/issues/114) features that required by us. We hope
to migrate to `iced-rs` once those features implemented by them.

### imgui-rs

The rust bindings to imgUI is also popular and supporting both platforms. But they do not have any
example provided to compile it as WASM binaries. It exposing a way to control GL. But do not have any example
about how to create GL windows using WASM.  It using the immediate mode which using in games. But not ideal
for large projects. Also it built with C++ and ported to rust using a wrapper.

### egui

This is pure rust alternative to imgui. It also supporting both platforms and provide a way to control GL.
Also implements examples to access GL and those examples are platform agnostic. It is also using the immediate
mode design pattern which not ideal. They mentioned it as not stable library.

## Decision

As the `egui` is providing features that we required, we start our developments using the `egui`. But it using the
immediate mode. So we have to move `iced-rs` once they implemented our required features. I know migrating from
immediate mode to Elm is difficult. But we can test all other functionalities.
