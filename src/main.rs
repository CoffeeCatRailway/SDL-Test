#![allow(non_snake_case)]

use std::error::Error;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::timer;

const FPS: u64 = 60;
const WAIT_TIME: u64 = 1000 / FPS;

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl3::init()?;
	let video = sdl.video()?;
	
	let title = "rust-sdl2 demo: Video";
	let window = video.window(title, 800, 600)
		.position_centered()
		// .opengl()
		.resizable()
		.build()?;
	
	let mut canvas = window.into_canvas();
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();
	canvas.present();
	let mut event_pump = sdl.event_pump()?;
	
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	'main: loop {
		let startTick = timer::ticks();
		
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				Event::MouseMotion { x, y, .. } => println!("X: {}, Y: {}", x, y),
				_ => {},
			}
		}
		
		canvas.clear();
		canvas.present();
		
		fps += 1;
		if startTick > lastTick + 1000 {
			let window = canvas.window_mut();
			let newTitle = format!("{} - FPS: {}", title, fps);
			window.set_title(&newTitle)?;
			
			lastTick = startTick;
			fps = 0;
		}
		
		let elapsedTicks = timer::ticks() - startTick;
		let waitTime = WAIT_TIME.saturating_sub(elapsedTicks);
		if waitTime > 0 {
			timer::delay(waitTime as u32);
		}
	}
	
	Ok(())
}
