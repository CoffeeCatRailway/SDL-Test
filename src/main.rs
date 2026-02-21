#![allow(non_snake_case)]

use std::error::Error;
use glow::HasContext;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::timer;
use sdl3::video::GLProfile;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;

const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

fn createSdl3Context(title: &str) -> Result<(
	glow::Context,
	sdl3::video::Window,
	sdl3::EventPump,
	sdl3::video::GLContext,
), Box<dyn Error>> {
	let sdl = sdl3::init()?;
	let video = sdl.video()?;
	let glAttributes = video.gl_attr();
	
	glAttributes.set_context_profile(GLProfile::Core);
	glAttributes.set_context_version(3, 3);
	// glAttributes.set_context_flags().forward_compatible().set();
	
	let window = video.window(title, WIN_WIDTH, WIN_HEIGHT)
					  .opengl()
					  .resizable()
					  .build()?;
	let glContext = window.gl_create_context()?;
	let gl = unsafe {
		glow::Context::from_loader_function(|s| {
			if let Some(ptr) = video.gl_get_proc_address(s) {
				// println!("{}", s);
				ptr as *const _
			} else {
				std::ptr::null()
			}
		})
	};
	let eventLoop = sdl.event_pump()?;
	
	Ok((gl, window, eventLoop, glContext))
}

fn main() -> Result<(), Box<dyn Error>> {
	// initialize
	let title = "SDL3-Glow Test";
	let (gl, mut window, mut eventLoop, _glContext) = createSdl3Context(title)?;
	
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	// let mut dt: f32 = OPTIMAL_WAIT_TIME as f32 / 1000.0;
	'main: loop {
		let startTick = timer::ticks();
		
		// events
		for event in eventLoop.poll_iter() {
			match event {
				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				// Event::MouseMotion { x, y, .. } => println!("X: {}, Y: {}", x, y),
				// Event::Window { win_event, ..} => {
				// 	match win_event {
				// 		WindowEvent::Resized(width, height) => {
				// 			windowSize.x = width;
				// 			windowSize.y = height;
				// 		},
				// 		_ => {},
				// 	}
				// }
				_ => {},
			}
		}
		
		// update
		// ...
		
		// render
		unsafe {
			gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			gl.clear_color(0.27, 0.59, 0.27, 1.0);
		}
		
		window.gl_swap_window();
		
		// fps counter
		fps += 1;
		if startTick > lastTick + 1000 {
			let newTitle = format!("{} - FPS: {}", title, fps);
			window.set_title(&newTitle)?;
			
			lastTick = startTick;
			fps = 0;
		}
		
		// timing
		let elapsedTicks = timer::ticks() - startTick;
		let waitTime = OPTIMAL_WAIT_TIME.saturating_sub(elapsedTicks);
		// dt = waitTime as f32 / 1000.0;
		if waitTime > 0 {
			// println!("{}", waitTime);
			timer::delay(waitTime as u32);
		}
	}
	
	// destroy
	// ...
	
	Ok(())
}

// fn main() -> Result<(), Box<dyn Error>> {
//     let sdl = sdl3::init()?;
// 	let video = sdl.video()?;
//
// 	let mut windowSize = Point::new(800, 600);
//
// 	let title = "SDL3 Test";
// 	let window = video.window(title, windowSize.x as u32, windowSize.y as u32)
// 		.position_centered()
// 		.resizable()
// 		.build()?;
//
// 	let mut canvas = window.into_canvas();
// 	let mut events = sdl.event_pump()?;
//
// 	let mut t = 0.0;
//
// 	let mut fps: u64 = 0;
// 	let mut lastTick: u64 = 0;
// 	let mut dt: f32 = OPTIMAL_WAIT_TIME as f32 / 1000.0;
// 	'main: loop {
// 		let startTick = timer::ticks();
//
// 		// events
// 		for event in events.poll_iter() {
// 			match event {
// 				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
// 				// Event::MouseMotion { x, y, .. } => println!("X: {}, Y: {}", x, y),
// 				Event::Window { win_event, ..} => {
// 					match win_event {
// 						WindowEvent::Resized(width, height) => {
// 							windowSize.x = width;
// 							windowSize.y = height;
// 						},
// 						_ => {},
// 					}
// 				}
// 				_ => {},
// 			}
// 		}
//
// 		// update
// 		t += dt;
//
// 		// render
// 		canvas.set_draw_color(Color::BLACK);
// 		canvas.clear();
//
// 		canvas.set_draw_color(Color::WHITE);
// 		let s = windowSize.x.min(windowSize.y) / 4;
// 		let p = windowSize / 2;
// 		let (xo, yo) = {
// 			let x = (t * 1.5).sin() * (s as f32 / 2.0);
// 			let y = (t * 1.5).cos() * (s as f32 / 2.0);
// 			(x as i32, y as i32)
// 		};
// 		canvas.draw_rect(Rect::new(p.x - s / 2 + xo, p.y - s / 2 + yo, s as u32, s as u32))?;
//
// 		canvas.set_draw_color(Color::GREEN);
// 		canvas.draw_line(Point::new(0, 0), windowSize)?;
//
// 		canvas.present();
//
// 		// fps counter
// 		fps += 1;
// 		if startTick > lastTick + 1000 {
// 			let window = canvas.window_mut();
// 			let newTitle = format!("{} - FPS: {}", title, fps);
// 			window.set_title(&newTitle)?;
//
// 			lastTick = startTick;
// 			fps = 0;
// 		}
//
// 		// timing
// 		let elapsedTicks = timer::ticks() - startTick;
// 		let waitTime = OPTIMAL_WAIT_TIME.saturating_sub(elapsedTicks);
// 		dt = waitTime as f32 / 1000.0;
// 		if waitTime > 0 {
// 			// println!("{}", waitTime);
// 			timer::delay(waitTime as u32);
// 		}
// 	}
//
// 	Ok(())
// }
