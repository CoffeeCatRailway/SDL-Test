#![allow(non_snake_case)]

use std::error::Error;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::timer;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;

// fn rand01() -> f32 {
// 	rand::random::<f32>()
// }

// pub fn loadAndCompileShader<'a, P: AsRef<Path>>(kind: ShaderKind, path: P) -> Vec<u8> {
// 	let pathString = path.as_ref().display().to_string();
// 	println!("Loading GLSL shader from {}", pathString);
// 	let source = std::fs::read_to_string(path).expect("Couldn't read shader from file");
//
// 	let compiler = shaderc::Compiler::new().unwrap();
// 	compiler.compile_into_spirv(&source, kind, &pathString, "main", None)
// 			.expect("Couldn't compile shader.")
// 			.as_binary_u8()
// 			.to_vec()
// }

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl3::init()?;
	let video = sdl.video()?;
	
	let title = "SDL3 Test";
	let window = video.window(title, 800, 600)
		.position_centered()
		.resizable()
		.build()?;
	
	let mut canvas = window.into_canvas();
	let mut events = sdl.event_pump()?;
	
	let mut p = Point::new(800, 600);
	
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	'main: loop {
		let startTick = timer::ticks();
		
		// events
		for event in events.poll_iter() {
			match event {
				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				// Event::MouseMotion { x, y, .. } => println!("X: {}, Y: {}", x, y),
				Event::Window { win_event, ..} => {
					match win_event {
						WindowEvent::Resized(width, height) => {
							p.x = width;
							p.y = height;
						},
						_ => {},
					}
				}
				_ => {},
			}
		}
		
		// update
		
		// render
		canvas.set_draw_color(Color::BLACK);
		canvas.clear();
		
		canvas.set_draw_color(Color::GREEN);
		canvas.draw_line(Point::new(0, 0), p)?;
		
		canvas.present();
		
		// fps counter
		fps += 1;
		if startTick > lastTick + 1000 {
			let window = canvas.window_mut();
			let newTitle = format!("{} - FPS: {}", title, fps);
			window.set_title(&newTitle)?;
			
			lastTick = startTick;
			fps = 0;
		}
		
		// timing
		let elapsedTicks = timer::ticks() - startTick;
		let waitTime = OPTIMAL_WAIT_TIME.saturating_sub(elapsedTicks);
		if waitTime > 0 {
			// println!("{}", waitTime);
			timer::delay(waitTime as u32);
		}
	}
	
	Ok(())
}
