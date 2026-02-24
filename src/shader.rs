#![allow(non_snake_case)]
#![allow(unused)]

use std::sync::Arc;
use glow::{HasContext, Program};
use log::{error, info, warn};

type GlowShader = glow::Shader;

const F_NONE: u8 = 		0b00000000;
const F_DESTROYED: u8 = 0b00000001;
const F_LINKED: u8 = 	0b00000010;

pub enum ShaderType {
	Vertex,
	Fragment,
	Compute,
}

pub struct Shader {
	gl: Arc<glow::Context>,
	program: Program,
	flags: u8,
	shaders: Vec<GlowShader>,
}

impl Shader {
	fn destroyed(&self) -> bool {
		self.flags & F_DESTROYED != 0
	}
	
	fn linked(&self) -> bool {
		self.flags & F_LINKED != 0
	}
	
	pub fn new(gl: Arc<glow::Context>) -> Self {
		unsafe {
			info!("Creating shader program");
			let program = gl.create_program().expect("Failed to create shader program");
			Shader {
				gl,
				program,
				flags: F_NONE,
				shaders: Vec::new(),
			}
		}
	}
	
	pub fn attachFromSource(mut self, stype: ShaderType, source: &str) -> Self {
		if self.destroyed() {
			panic!("Shader program was destroyed before linking!");
		}
		if self.linked() {
			error!("Shader program {} is already linked! Unable to attach other shaders!", self.program.0);
			return self;
		}
		unsafe {
			let (typeStr, typeGlow) = match stype {
				ShaderType::Vertex => ("vertex", glow::VERTEX_SHADER),
				// glow::TESS_CONTROL_SHADER => "tess-control",
				// glow::TESS_EVALUATION_SHADER => "tess-evaluation",
				// glow::GEOMETRY_SHADER => "geometry",
				ShaderType::Fragment => ("fragment", glow::FRAGMENT_SHADER),
				ShaderType::Compute => ("compute", glow::COMPUTE_SHADER),
			};
			info!("Attaching {} shader to program {}...", typeStr, self.program.0);
			
			let shader = self.gl.create_shader(typeGlow).expect(format!("Failed to create shader of type '{}'", typeStr).as_str());
			self.gl.shader_source(shader, source);
			self.gl.compile_shader(shader);
			
			if !self.gl.get_shader_compile_status(shader) {
				let error = self.gl.get_shader_info_log(shader);
				panic!("Failed to compile shader: {error}");
			}
			self.gl.attach_shader(self.program, shader);
			self.shaders.push(shader);
		}
		self
	}
	
	pub fn link(mut self) -> Self {
		if self.destroyed() {
			panic!("Shader program was destroyed before linking!");
		}
		if self.linked() {
			error!("Shader program {} is already linked!", self.program.0);
			return self;
		}
		unsafe {
			info!("Linking shader program {}...", self.program.0);
			
			self.gl.bind_frag_data_location(self.program, glow::COLOR_ATTACHMENT0, "o_color");
			self.gl.link_program(self.program);
			if !self.gl.get_program_link_status(self.program) {
				let error = self.gl.get_program_info_log(self.program);
				panic!("Failed to link shader: {}", error);
			}
			
			for shader in self.shaders.iter() {
				self.gl.detach_shader(self.program, *shader);
				self.gl.delete_shader(*shader);
			}
			self.shaders = Vec::new(); // Clear and deallocate
			self.flags |= F_LINKED;
		}
		self
	}
	
	pub fn program(&self) -> Option<&Program> {
		if self.destroyed() || !self.linked() {
			None
		} else {
			Some(&self.program)
		}
	}
	
	pub fn bind(&self) {
		if self.destroyed() {
			error!("Shader program was destroyed before binding!");
			return;
		}
		if !self.linked() {
			error!("Shader program {} is not linked!", self.program.0);
			return;
		}
		unsafe {
			self.gl.use_program(Some(self.program));
		}
	}
	
	pub fn delete(&mut self) {
		if self.destroyed() {
			return;
		}
		unsafe {
			warn!("Deleting shader program {}...", self.program.0);
			self.gl.delete_program(self.program);
			self.flags |= F_DESTROYED;
		}
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.delete();
	}
}
