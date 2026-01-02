use imgui_sdl3::ImGuiSdl3;
use imgui_sdl3::utils::create_texture;
use sdl3::{event::Event, gpu::*, pixels::Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize SDL and its video subsystem
    let mut sdl = sdl3::init()?;
    let video_subsystem = sdl.video()?;

    // create a new window
    let window = video_subsystem
        .window("Hello imgui-rs!", 1280, 720)
        .position_centered()
        .resizable()
        .build()?;

    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

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
    let mut event_pump = sdl.event_pump()?;

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

            // Load a PNG from disk
            let img = image::load_from_memory(include_bytes!("./assets/rust-logo.png"))?.to_rgba8();
            let (w, h) = (img.width() as u32, img.height() as u32);
            let pixels = img.into_raw(); // RGBA8 pixel bytes

            // Create a GPU Texture and a Sampler from the device
            let copy_commands = device.acquire_command_buffer()?;
            let copy_pass = device.begin_copy_pass(&copy_commands)?;

            let texture = create_texture(&device, &copy_pass, &pixels, w, h)?;

            device.end_copy_pass(copy_pass);
            copy_commands.submit()?;

            let sampler: sdl3::gpu::Sampler = device.create_sampler(
                SamplerCreateInfo::new()
                    .with_min_filter(Filter::Linear)
                    .with_mag_filter(Filter::Linear)
                    .with_mipmap_mode(SamplerMipmapMode::Linear)
                    .with_address_mode_u(SamplerAddressMode::Repeat)
                    .with_address_mode_v(SamplerAddressMode::Repeat)
                    .with_address_mode_w(SamplerAddressMode::Repeat),
            )?;

            // Register the texture and get a TextureId
            let rust_logo_tex = imgui.push_texture(texture, sampler);

            imgui.render(
                &mut sdl,
                &device,
                &window,
                &event_pump,
                &mut command_buffer,
                &color_targets,
                |ui| {
                    ui.image_button("##", rust_logo_tex, [w as f32, h as f32]);
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
