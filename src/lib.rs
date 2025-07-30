pub mod platform;
pub mod renderer;
pub mod utils;
use platform::Platform;
use renderer::Renderer;
use sdl3::gpu::*;

pub struct ImGuiSdl3 {
    imgui_context: imgui::Context,
    platform: Platform,
    renderer: Renderer,
}

impl ImGuiSdl3 {
    pub fn new<T>(device: &sdl3::gpu::Device, window: &sdl3::video::Window, ctx_configure: T) -> Self
    where
        T: Fn(&mut imgui::Context),
    {
        let mut imgui_context = imgui::Context::create();
        ctx_configure(&mut imgui_context);

        let platform = Platform::new(&mut imgui_context);
        let renderer = Renderer::new(device, window, &mut imgui_context).unwrap();

        Self {
            imgui_context,
            platform,
            renderer,
        }
    }

    pub fn handle_event(&mut self, event: &sdl3::event::Event) {
        self.platform.handle_event(&mut self.imgui_context, event);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render<T>(
        &mut self,
        sdl_context: &mut sdl3::Sdl,
        device: &sdl3::gpu::Device,
        window: &sdl3::video::Window,
        event_pump: &sdl3::EventPump,
        command_buffer: &mut CommandBuffer,
        color_targets: &[ColorTargetInfo],
        mut draw_callback: T,
    ) where
        T: FnMut(&mut imgui::Ui),
    {
        self.platform
            .prepare_frame(sdl_context, &mut self.imgui_context, window, event_pump);

        let ui = self.imgui_context.new_frame();
        draw_callback(ui);

        self.renderer
            .render(device, command_buffer, color_targets, &mut self.imgui_context)
            .unwrap();
    }
}
