use std::sync::Arc;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

mod chip;
mod compiler;

use chip::*;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App<'win> {
    window: Option<Arc<Window>>,
    chip: Chip,
    pixels: Option<Pixels<'win>>,
    last_cycle: Option<Instant>,
    cycle_delay: Duration,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(LogicalSize::new(640, 320))
                        .with_title("CHIP-8"),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let surface = SurfaceTexture::new(640, 320, window.clone());
        self.pixels = Some(Pixels::new(64, 32, surface).unwrap());

        let args: Vec<String> = std::env::args().collect();

        self.chip.load_rom_into_memory(args[1].clone());

        self.last_cycle = Some(Instant::now());
        self.cycle_delay = Duration::from_millis(2);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let last = self.last_cycle.get_or_insert(now);

                if now.duration_since(*last) >= self.cycle_delay {
                    self.last_cycle = Some(now);
                    for _ in 0..10 {
                        self.chip.cycle();
                    }
                }

                self.chip.tick_timers();

                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();
                    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        let color = if self.chip.display_memory[i] != 0 {
                            [0xff, 0xff, 0xff, 0xff]
                        } else {
                            [0x00, 0x00, 0x00, 0xff]
                        };
                        pixel.copy_from_slice(&color);
                    }
                    pixels.render().unwrap();
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == winit::event::ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Digit1) => self.chip.input[0x1] = pressed,
                    PhysicalKey::Code(KeyCode::Digit2) => self.chip.input[0x2] = pressed,
                    PhysicalKey::Code(KeyCode::Digit3) => self.chip.input[0x3] = pressed,
                    PhysicalKey::Code(KeyCode::Digit4) => self.chip.input[0xc] = pressed,
                    PhysicalKey::Code(KeyCode::KeyQ) => self.chip.input[0x4] = pressed,
                    PhysicalKey::Code(KeyCode::KeyW) => self.chip.input[0x5] = pressed,
                    PhysicalKey::Code(KeyCode::KeyE) => self.chip.input[0x6] = pressed,
                    PhysicalKey::Code(KeyCode::KeyR) => self.chip.input[0xd] = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.chip.input[0x7] = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.chip.input[0x8] = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.chip.input[0x9] = pressed,
                    PhysicalKey::Code(KeyCode::KeyF) => self.chip.input[0xe] = pressed,
                    PhysicalKey::Code(KeyCode::KeyZ) => self.chip.input[0xa] = pressed,
                    PhysicalKey::Code(KeyCode::KeyX) => self.chip.input[0x0] = pressed,
                    PhysicalKey::Code(KeyCode::KeyC) => self.chip.input[0xb] = pressed,
                    PhysicalKey::Code(KeyCode::KeyV) => self.chip.input[0xf] = pressed,
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

impl App<'_> {
    fn reset(&mut self, rom: String) {
        self.chip = Chip::new();
        self.chip.load_rom_into_memory(rom);
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
