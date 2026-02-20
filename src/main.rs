#![allow(non_snake_case)]

use std::error::Error;
use std::path::Path;
use sdl3::event::Event;
use sdl3::gpu::{BufferRegion, BufferUsageFlags, ColorTargetDescription, ColorTargetInfo, Device, GraphicsPipelineTargetInfo, LoadOp, PrimitiveType, ShaderFormat, ShaderStage, StorageBufferReadWriteBinding, StoreOp, TransferBufferLocation, TransferBufferUsage};
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::timer;
use shaderc::ShaderKind;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;

#[repr(C)]
#[derive(Copy, Clone)]
struct Particle {
	pos: [f32; 2],
	vel: [f32; 2],
	color: [f32; 4],
}

const PARTICLE_COUNT: u32 = 1000;

fn rand01() -> f32 {
	rand::random::<f32>()
}

pub fn loadAndCompileShader<'a, P: AsRef<Path>>(kind: ShaderKind, path: P) -> Vec<u8> {
	let pathString = path.as_ref().display().to_string();
	println!("Loading GLSL shader from {}", pathString);
	let source = std::fs::read_to_string(path).expect("Couldn't read shader from file");
	
	let compiler = shaderc::Compiler::new().unwrap();
	compiler.compile_into_spirv(&source, kind, &pathString, "main", None)
			.expect("Couldn't compile shader.")
			.as_binary_u8()
			.to_vec()
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl3::init()?;
	let video = sdl.video()?;
	
	let title = "rust-sdl2 demo: Video";
	let window = video.window(title, 800, 600)
		.position_centered()
		.resizable()
		.build()?;
	
	let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;
	
	// create buffer
	let particles: Vec<Particle> = (0..PARTICLE_COUNT).map(|_| Particle {
		pos: [rand01() * 2.0 - 1.0, rand01() * 2.0 - 1.0],
		vel: [0.0, -0.1],
		color: [rand01(), rand01(), rand01(), 1.0],
	}).collect();
	
	let bufferSize = (PARTICLE_COUNT as usize * size_of::<Particle>()) as u32;
	let particleBuffer = device.create_buffer().with_size(bufferSize).with_usage(BufferUsageFlags::COMPUTE_STORAGE_WRITE).build()?;
	
	// upload to gpu
	let upload = device.create_transfer_buffer().with_size(bufferSize).with_usage(TransferBufferUsage::UPLOAD).build()?;
	{
		let mut map = upload.map::<Particle>(&device, true);
		map.mem_mut().copy_from_slice(&particles);
		map.unmap();
		
		let copyCmd = device.acquire_command_buffer()?;
		let copyPass = device.begin_copy_pass(&copyCmd)?;
		copyPass.upload_to_gpu_buffer(
			TransferBufferLocation::new().with_offset(0).with_transfer_buffer(&upload),
			BufferRegion::new().with_offset(0).with_size(bufferSize).with_buffer(&particleBuffer),
			true,
		);
		device.end_copy_pass(copyPass);
		copyCmd.submit()?;
	}
	
	// shaders
	let pipeline = device.create_graphics_pipeline()
						 .with_primitive_type(PrimitiveType::PointList)
						 .with_vertex_shader(
							 &device.create_shader()
									.with_code(ShaderFormat::SPIRV, loadAndCompileShader(ShaderKind::Vertex, "shaders/particles.vert").as_slice(), ShaderStage::Vertex)
									.with_storage_buffers(1)
									.with_entrypoint(c"main")
									.build()?,
						 )
						 .with_fragment_shader(
							 &device.create_shader()
									.with_code(ShaderFormat::SPIRV, loadAndCompileShader(ShaderKind::Fragment, "shaders/particles.frag").as_slice(), ShaderStage::Fragment)
									.with_entrypoint(c"main")
									.build()?
						 )
						 .with_target_info(GraphicsPipelineTargetInfo::new()
							 .with_color_target_descriptions(&[ColorTargetDescription::new()
								 .with_format(device.get_swapchain_texture_format(&window))])
						 ).build()?;
	
	let computePipeline = device.create_compute_pipeline()
								.with_code(ShaderFormat::SPIRV, loadAndCompileShader(ShaderKind::Compute, "shaders/particles.comp").as_slice())
								.with_entrypoint(c"main")
								.with_readwrite_storage_buffers(1)
								.with_thread_count(64, 1, 1)
								.build()?;
	
	let mut canvas = window.into_canvas();
	let mut events = sdl.event_pump()?;
	
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	'main: loop {
		let startTick = timer::ticks();
		
		// events
		for event in events.poll_iter() {
			match event {
				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				// Event::MouseMotion { x, y, .. } => println!("X: {}, Y: {}", x, y),
				_ => {},
			}
		}
		
		// update
		let computeCmd = device.acquire_command_buffer()?;
		let binding = StorageBufferReadWriteBinding::new()
			.with_buffer(&particleBuffer)
			.with_cycle(false);
		let computePass = device.begin_compute_pass(&computeCmd, &[], &[binding])?;
		computePass.bind_compute_pipeline(&computePipeline);
		computePass.dispatch(PARTICLE_COUNT.div_ceil(64), 1, 1);
		device.end_compute_pass(computePass);
		computeCmd.submit()?;
		
		// render
		let mut drawCmd = device.acquire_command_buffer()?;
		if let Ok(swapchain) = drawCmd.wait_and_acquire_swapchain_texture(canvas.window()) {
			let colorTarget = ColorTargetInfo::default()
				.with_texture(&swapchain)
				.with_load_op(LoadOp::CLEAR)
				.with_store_op(StoreOp::STORE)
				.with_clear_color(Color::RGB(10, 10, 30));
			let pass = device.begin_render_pass(&drawCmd, &[colorTarget], None)?;
			
			pass.bind_graphics_pipeline(&pipeline);
			pass.bind_vertex_storage_buffers(0, &[particleBuffer.clone()]);
			pass.draw_primitives(PARTICLE_COUNT as usize, 1, 0, 0);
			
			device.end_render_pass(pass);
			drawCmd.submit()?;
		} else {
			drawCmd.cancel();
		}
		
		// canvas.set_draw_color(Color::BLACK);
		// canvas.clear();
		//
		// canvas.set_draw_color(Color::GREEN);
		// canvas.draw_line(sdl3::rect::Point::new(0, 0), sdl3::rect::Point::new(800, 600))?;
		//
		// canvas.present();
		
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
