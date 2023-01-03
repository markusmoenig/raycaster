#![deny(clippy::all)]
#![forbid(unsafe_code)]

use raycaster::prelude::*;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum WidgetKey {
    Escape,
    Return,
    Delete,
    Up,
    Right,
    Down,
    Left,
    Space,
    Tab
}

pub mod prelude {
    pub use crate::WidgetKey;
}

use raycaster::{self, raycaster::Raycaster};

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use tao::{
    dpi::PhysicalPosition,
    dpi::LogicalSize,
    event::{Event, DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    menu::{MenuBar, MenuItem},
    window::WindowBuilder,
    keyboard::{Key},
};

use std::time::{SystemTime, UNIX_EPOCH};

use std::path::PathBuf;
use std::fs::File;

/// Get the time in ms
fn get_time() -> u128 {
    let stop = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
        stop.as_millis()
}

/// Load an image from a file
fn load(file_name: &PathBuf) -> (Vec<u8>, u32, u32) {

    let decoder = png::Decoder::new(File::open(file_name).unwrap());
    if let Ok(mut reader) = decoder.read_info() {
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..info.buffer_size()];

        return (bytes.to_vec(), info.width, info.height);
    }
    (vec![], 0 , 0)
}

fn main() -> Result<(), Error> {

    let width     : usize = 1280;
    let height    : usize = 800;

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = {
        let mut file_menu = MenuBar::new();
        file_menu.add_native_item(MenuItem::Quit);

        let mut menu = MenuBar::new();
        menu.add_submenu("File", true, file_menu);

        let size = LogicalSize::new(width as f64, height as f64);
        WindowBuilder::new()
            .with_title("Raycaster Demo")
            .with_menu(menu)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width as u32, height as u32, surface_texture)?
    };

    // Load the tilemap

    let (tilemap, tilemap_width, tilemap_height) = load(&PathBuf::from("resources/tilemap.png"));

    // Init the world map

    let mut world = WorldMap::new();
    let image_id = world.add_image(tilemap, tilemap_width, tilemap_height);

    let calc_tile_rect = |x: usize, y: usize, tile_size: usize| -> (usize, usize, usize, usize) {
        (x * tile_size * 4, y * tile_size * tilemap_width as usize * 4, tile_size, tile_size)
    };

    let wall = Tile::textured(image_id, calc_tile_rect(20, 4, 24));

    world.set_wall(-5, 5, wall.clone());

    let mut caster = Raycaster::new(world, width, height);

    let mut coords = PhysicalPosition::new(0.0, 0.0);
    // let mut is_pressed = false;

    const GAME_TICK_IN_MS : u128 = 1000 / 30;

    let mut game_tick_timer : u128 = 0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                // Close events
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: KeyCode::Escape,
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                // Resize the window
                WindowEvent::Resized(size) => {
                    _ = pixels.resize_surface(size.width, size.height);
                    let scale = window.scale_factor() as u32;
                    _ = pixels.resize_buffer(size.width / scale, size.height / scale);
                    let width = size.width as usize / scale as usize;
                    let height = size.height as usize / scale as usize;
                    caster.resize(width, height);
                    window.request_redraw();
                }

                WindowEvent::CursorMoved { position, .. } => {
                    coords = position;
                }

                WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                    logical_key: key,
                    state: ElementState::Pressed,
                    ..
                    },
                ..
                } => {
                    // WARNING: Consider using `key_without_modifers()` if available on your platform.
                    // See the `key_binding` example
                    match key {
                        Key::ArrowLeft => {
                            caster.turn_left();
                        },
                        Key::ArrowRight => {
                            caster.turn_right();
                        },
                        Key::ArrowUp => {
                            caster.go_forward();
                        },
                        Key::ArrowDown => {
                            caster.go_backward();
                        },
                        _ => (),
                    }
                }
                WindowEvent::ModifiersChanged(_m) => {
                    // if ui.modifier_changed(m.shift_key(), m.control_key(), m.alt_key(), m.super_key()) {
                    //     window.request_redraw();
                    // }
                }
                _ => (),
            },

            // Update internal state and request a redraw
            Event::MainEventsCleared => {
                //window.request_redraw();

                let curr_time = get_time();

                // Game tick ?
                if curr_time > game_tick_timer + GAME_TICK_IN_MS {
                    // let start = get_time();
                    // let stop = get_time();
                    // println!("tick time {:?}", stop - start);
                    window.request_redraw();
                    game_tick_timer = curr_time;
                }
            }

            // Draw the current frame
            Event::RedrawRequested(_) => {

                let frame = pixels.get_frame_mut();
                caster.render(&mut frame[..]);

                if pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                }
            },

            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { /*delta,*/ .. } => {
                    //println!("mouse moved: {:?}", delta),
                    if let Some(_pixel_pos) = pixels.window_pos_to_pixel((coords.x as f32, coords.y as f32)).ok() {
                        // if is_pressed {
                        //     if ui.mouse_dragged(pixel_pos) {
                        //         window.request_redraw();
                        //     }
                        // } else
                        // if ui.mouse_hover(pixel_pos) {
                        //     window.request_redraw();
                        // }
                    }
                }
                DeviceEvent::Button {state, .. } => match state {
                    ElementState::Pressed => {
                        //println!("mouse button {} pressed", button);
                        if let Some(_pixel_pos) = pixels.window_pos_to_pixel((coords.x as f32, coords.y as f32)).ok() {
                            //is_pressed = true;
                            // if ui.mouse_down(pixel_pos) {
                            //     window.request_redraw();
                            // }
                        }
                    }
                    ElementState::Released => {
                        //println!("mouse button {} released", button),
                        if let Some(_pixel_pos) = pixels.window_pos_to_pixel((coords.x as f32, coords.y as f32)).ok() {
                            // is_pressed = false;
                            // if ui.mouse_up(pixel_pos) {
                            //     window.request_redraw();
                            // }
                        }
                    }
                    _ => (),
                },

                DeviceEvent::MouseWheel { delta, .. } => match delta {
                    // tao::event::MouseScrollDelta::LineDelta(x, y) => {
                    //     println!("mouse wheel Line Delta: ({},{})", x, y);
                    //     let pixels_per_line = 120.0;
                    //     let mut pos = window.outer_position().unwrap();
                    //     pos.x -= (x * pixels_per_line) as i32;
                    //     pos.y -= (y * pixels_per_line) as i32;
                    //     window.set_outer_position(pos)
                    // }
                    tao::event::MouseScrollDelta::PixelDelta(_p) => {
                        //println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
                        // if ui.mouse_wheel((p.x as isize, p.y as isize)) {
                        //     window.request_redraw();
                        //     //mouse_wheel_ongoing = true;
                        // }
                    }
                    _ => (),
                },
                _ => (),
            }
            _ => {}
        }
    });
}