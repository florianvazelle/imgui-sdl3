<div align="center">

# `imgui-sdl3`

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/florianvazelle/imgui-sdl3/nix.yml)
![GitHub License](https://img.shields.io/github/license/florianvazelle/imgui-sdl3)

Rust library that integrates Dear ImGui with SDL3.

</div>

## Features

This crate provides an SDL3 backend platform and renderer for imgui-rs.

- The backend platform handles window/input device events (based on [ghtalpo/imgui-sdl3-support](https://github.com/ghtalpo/imgui-sdl3-support)),
- The rendering backend use the SDL3 GPU API.

> For a canvas rendering backend, check out [masonjmj/imgui-rs-sdl3-renderer](https://github.com/masonjmj/imgui-rs-sdl3-renderer).

## Full demo

```rs
use imgui_sdl3::ImGuiSdl3;
use sdl3::{
    event::Event,
    gpu::{Device, ShaderFormat},
};

fn main() {
    // initialize SDL and its video subsystem
    let mut sdl = sdl3::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    // create a new window
    let window = video_subsystem
        .window("Hello imgui-rs!", 1280, 720)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let device = Device::new(ShaderFormat::SpirV, true)
        .unwrap()
        .with_window(&window)
        .unwrap();

    // create platform and renderer
    let mut imgui = ImGuiSdl3::new(&device, &window, |ctx| {
        // disable creation of files on disc
        ctx.set_ini_filename(None);
        ctx.set_log_filename(None);

        // setup platform and renderer, and fonts to imgui
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    });

    // start main loop
    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            // pass all events to imgui platfrom
            imgui.handle_event(&event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        imgui.render(&mut sdl, &device, &window, &event_pump, |ui| {
            // create imgui UI here
            ui.show_demo_window(&mut true);
        });
    }
}
```

## Development

The project use [`just`](https://just.systems/man/en/) as command runner.

To check all available recipes, run:
```
just
```

To run formatters:
```
just fmt
```
