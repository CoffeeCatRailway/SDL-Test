#![allow(non_snake_case)]

mod shader;

use std::error::Error;
use std::sync::Arc;
use glow::HasContext;
use log::info;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::timer;
use sdl3::video::GLProfile;
use crate::shader::{Shader, ShaderType};
use bytemuck::{Pod, Zeroable};
use rand::RngExt;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;

const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

const PARTICLE_COUNT: u32 = 1000;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Particle {
	pos: [f32; 2],
	vel: [f32; 2],
	color: [f32; 4],
}

fn rand01() -> f32 {
	rand::rng().random()
}

fn createSdl3Context(title: &str) -> Result<(
	glow::Context,
	sdl3::video::Window,
	sdl3::EventPump,
	sdl3::video::GLContext,
), Box<dyn Error>> {
	info!("Creating SDL3 context");
	let sdl = sdl3::init()?;
	let video = sdl.video()?;
	let glAttributes = video.gl_attr();
	
	glAttributes.set_context_profile(GLProfile::Core);
	glAttributes.set_context_version(4, 3);
	// glAttributes.set_context_flags().forward_compatible().set();
	
	info!("Creating window and GL context");
	let window = video.window(title, WIN_WIDTH, WIN_HEIGHT)
					  .opengl()
					  .resizable()
					  .build()?;
	let glContext = window.gl_create_context()?;
	let gl = unsafe {
		glow::Context::from_loader_function(|s| {
			if let Some(ptr) = video.gl_get_proc_address(s) {
				// info!("{}", s);
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
	tracing_subscriber::fmt::fmt()
		// .with_writer(Arc::new(file))
		.with_ansi(true)
		.with_target(false)
		.with_file(true)
		.with_line_number(true)
		.with_thread_names(true)
		.with_thread_ids(false)
		.compact()
		.with_max_level(tracing::Level::INFO)
		.init();
	
	info!("Hello, world!");
	// panic!("hi");
	
	// initialize
	let title = "SDL3-Glow Test";
	let (gl, mut window, mut eventLoop, _glContext) = createSdl3Context(title)?;
	let gl = Arc::new(gl);
	
	// create buffer
	let particles: Vec<Particle> = (0..PARTICLE_COUNT).map(|_| Particle {
		pos: [rand01() * 2.0 - 1.0, rand01() * 2.0 - 1.0],
		vel: [0.0, -0.1],
		color: [rand01(), rand01(), rand01(), 1.0],
	}).collect();
	
	// let bufferSize = (PARTICLE_COUNT as usize * size_of::<Particle>()) as u32;
	let ssbo = unsafe {
		let ssbo = gl.create_named_buffer()?;
		gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(ssbo));
		gl.named_buffer_data_u8_slice(ssbo, bytemuck::cast_slice(particles.as_slice()), glow::DYNAMIC_DRAW);
		gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);
		ssbo
	};
	
	let vao = unsafe {
		let vao = gl.create_vertex_array()?;
		gl.bind_vertex_array(Some(vao));
		vao
	};
	
	// shaders
	let mut shader = Shader::new(gl.clone())
		.attachFromSource(ShaderType::Vertex, include_str!("../shaders/particles.vert"))
		.attachFromSource(ShaderType::Fragment, include_str!("../shaders/particles.frag"))
		.link();
	
	let mut compute = Shader::new(gl.clone())
		.attachFromSource(ShaderType::Compute, include_str!("../shaders/particles.comp"))
		.link();
	
	unsafe {
		gl.enable(glow::PROGRAM_POINT_SIZE);
	}
	
	info!("Starting main loop");
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	// let mut dt: f32 = OPTIMAL_WAIT_TIME as f32 / 1000.0;
	'main: loop {
		let startTick = timer::ticks();
		
		// events
		for event in eventLoop.poll_iter() {
			match event {
				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					info!("Exiting main loop");
					break 'main
				},
				Event::Window { win_event, .. } => match win_event {
					WindowEvent::Resized(w, h) => unsafe {
						gl.viewport(0, 0, w.max(1), h.max(1));
					}
					_ => {},
				}
				_ => {},
			}
		}
		
		// update
		unsafe {
			gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);
			compute.bind();
			gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 0, Some(ssbo));
			
			gl.dispatch_compute(PARTICLE_COUNT.div_ceil(64), 1, 1);
			gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);
		}
		
		// render
		unsafe {
			gl.clear(glow::COLOR_BUFFER_BIT);
			gl.clear_color(0.27, 0.59, 0.27, 1.0);
			
			shader.bind();
			gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 0, Some(ssbo));
			gl.bind_vertex_array(Some(vao));
			gl.draw_arrays(glow::POINTS, 0, PARTICLE_COUNT as i32);
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
			// info!("{}", waitTime);
			timer::delay(waitTime as u32);
		}
	}
	
	// destroy
	info!("Cleaning up");
	compute.delete();
	shader.delete();
	unsafe {
		gl.delete_vertex_array(vao);
		gl.delete_buffer(ssbo);
	}
	
	Ok(())
}
