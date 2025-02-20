#[cfg(not(feature = "use_epi"))]
compile_error!("feature \"use_epi\" must be used");

use egui::{Visuals, epaint::Shadow};
use egui_backend::{
    egui,
    epi::{Frame, IntegrationInfo},
    get_frame_time, gl, sdl2,
    sdl2::event::Event,
    sdl2::video::GLProfile,
    sdl2::video::SwapInterval,
    DpiScaling, ShaderVersion, Signal,
};
use epi::backend::FrameData;
use std::{sync::Arc, time::Instant};
// Alias the backend to something less mouthful
use egui_sdl2_gl as egui_backend;
const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    // Let OpenGL know we are dealing with SRGB colors so that it
    // can do the blending correctly. Not setting the framebuffer
    // leads to darkened, oversaturated colors.
    gl_attr.set_framebuffer_srgb_compatible(true);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);

    // OpenGL 3.2 is the minimum that we will support.
    gl_attr.set_context_version(3, 2);

    let window = video_subsystem
        .window(
            "Demo: Egui backend for SDL2 + GL",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .opengl()
        .resizable()
        .build()
        .unwrap();

    // Create a window context
    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    // Enable vsync
    window
        .subsystem()
        .gl_set_swap_interval(SwapInterval::VSync)
        .unwrap();

    // Init egui stuff
    let (mut painter, mut egui_state) =
        egui_backend::with_sdl2(&window, ShaderVersion::Default, DpiScaling::Custom(2.5));
    let mut app = egui_demo_lib::DemoWindows::default();
    let egui_ctx = egui::Context::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let start_time = Instant::now();
    let repaint_signal = Arc::new(Signal::default());
    
    egui_ctx.set_visuals(Visuals {
        window_shadow: Shadow::NONE,
        ..Default::default()
        });

    'running: loop {
        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());
        // Begin frame
        let frame_time = get_frame_time(start_time);
        let frame = Frame::new(FrameData {
            info: IntegrationInfo {
                web_info: None,
                cpu_usage: Some(frame_time),
                native_pixels_per_point: Some(egui_state.native_pixels_per_point),
                prefer_dark_mode: None,
                name: "egui + sdl2 + gl",
            },
            output: Default::default(),
            repaint_signal: repaint_signal.clone(),
        });

        app.ui(&egui_ctx);
        let full_output = egui_ctx.end_frame();
        // Process ouput
        egui_state.process_output(&window, &full_output.platform_output);
        // Quite if needed.
        if frame.take_app_output().quit {
            break 'running;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {
                    // Process input event
                    egui_state.process_input(&window, event, &mut painter);
                }
            }
        }

        // For default dpi scaling only, Update window when the size of resized window is very small (to avoid egui::CentralPanel distortions).
        // if egui_ctx.used_size() != painter.screen_rect.size() {
        //     println!("resized.");
        //     let _size = egui_ctx.used_size();
        //     let (w, h) = (_size.x as u32, _size.y as u32);
        //     window.set_size(w, h).unwrap();
        // }

        let paint_jobs = egui_ctx.tessellate(full_output.shapes);

        // An example of how OpenGL can be used to draw custom stuff with egui
        // overlaying it:
        // First clear the background to something nice.
        unsafe {
            // Clear the screen to green
            gl::ClearColor(0.3, 0.6, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        painter.paint_jobs(None, &paint_jobs, &full_output.textures_delta);
        window.gl_swap_window();
    }
}
