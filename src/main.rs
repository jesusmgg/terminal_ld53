mod audio;
mod game;
mod input;
mod renderer;
mod resources;

use std::time::Instant;

use game::game_state::GameState;
use renderer::render_state::RenderState;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "terminal_ld53";
const WINDOW_WIDTH: u16 = 1600;
const WINDOW_HEIGHT: u16 = 900;

fn main() {
    // Use pollster to block thread while game loop runs
    pollster::block_on(game_loop());
}

/// Main game loop
async fn game_loop<'a>() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::PhysicalSize {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        })
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let mut render_state = RenderState::new(window).await;
    let mut game_state = GameState::new(&event_loop, &mut render_state).await;

    let mut last_render_time = Instant::now();

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Game state input
        game_state.input(&event, &render_state.window);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == render_state.window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,

                WindowEvent::Resized(physical_size) => {
                    render_state.resize(*physical_size);
                }

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    render_state.resize(**new_inner_size);
                }

                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == render_state.window.id() => {
                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                // Game state update
                // TODO: decouple game logic update from rendering/input
                game_state.update(&mut render_state, dt);
                game_state.ui(&mut render_state);
                render_state.update(dt);

                // Game state render inside render state render function
                match render_state.render(&mut game_state) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                render_state.window.request_redraw();
            }
            _ => {}
        }
    });
}
