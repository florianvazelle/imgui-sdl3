use glam::Mat4;
use imgui_sdl3::{ImGuiSdl3, imoguizmo};
use sdl3::{event::Event, gpu::*, pixels::Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let device = Device::new(ShaderFormat::SPIRV, true)
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

    let mut view = Mat4::from_rotation_x(90.0);
    let proj = Mat4::IDENTITY;

    // start main loop
    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            // pass all events to imgui platform
            imgui.handle_event(&event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        let mut command_buffer = device.acquire_command_buffer()?;

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&window) {
            let color_targets = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(128, 128, 128))];

            imgui.render(
                &mut sdl,
                &device,
                &window,
                &event_pump,
                &mut command_buffer,
                &color_targets,
                |ui| {
                    imoguizmo::begin_frame(ui, true, || {
                        if let Some(new_view) = imoguizmo::draw_gizmo(ui, view.into(), proj.into(), 1.0) {
                            view = new_view.into();
                        }
                    });
                },
            );

            command_buffer.submit()?;
        } else {
            println!("Swapchain unavailable, cancel work");
            command_buffer.cancel();
        }
    }

    Ok(())
}
