pub mod ui {
    use raylib::prelude::*;

	use gambling_simulator::{SlotBoard, SlotGame, SymbolType};

	#[derive(PartialEq)]
	pub enum BoardSpinningState {
		Spinning,
		Stopping,
		Stopped
	}
	pub struct UI {
		slot_size: Vector2,
		win_dimensions: (i32, i32),
		margin: i32,
		padding: i32,

		pub last_money: f64,

		pub game: SlotGame,
		pub reels: Vec<Reel>,
		pub spinning: BoardSpinningState,
		stopped_spinning: bool,

		pub displaying_pay_lines: bool,

		pub rl: RaylibHandle,
		thread: RaylibThread,

		run_time: f32,
		run_time_loc: i32,
		rainbow_shader: Shader,
		font: Font
	}

	fn get_text_size(text: String, font: &Font, font_size: f32) -> raylib::ffi::Vector2 {
		let str = std::ffi::CString::new(text).unwrap();
		unsafe {
			raylib::ffi::MeasureTextEx(
				**font,
				str.as_ptr(),
				font_size,
				font_size/10.0
			)
		}
	}

	impl UI {
		pub fn new(slot_size: Vector2, margin: i32, padding: i32, board_dimensions: (usize, usize)) -> UI {
			let mut game = SlotGame::new(board_dimensions.0, board_dimensions.1);

			let win_dimensions: (i32, i32) = (
				slot_size.x as i32*board_dimensions.0 as i32 + margin*2 + padding*(1 + board_dimensions.0 as i32),
				slot_size.x as i32*board_dimensions.1 as i32 + margin*2 + padding*(1 + board_dimensions.1 as i32) + 100 // Add space for game information
			);
			let (mut rl, thread) = raylib::init()
				.size(
					win_dimensions.0,
					win_dimensions.1
				)
				.title("Gambling Simulator")
				.build();

			let rainbow_shader = rl.load_shader(&thread, None, Some("src/rainbow.fs")).unwrap();
			let run_time_loc = rainbow_shader.get_shader_location("runTime");

			let font = rl.load_font(&thread, "assets/jupiter_crash.png").unwrap();
			

			rl.set_window_icon(Image::load_image("assets/slot-machine.png").unwrap());

			let mut reels = Vec::new();
			for x in 0..board_dimensions.0 {
				let mut reel_symbols = Vec::new();
				
				reel_symbols.push(game.board.rand_symbol());
				for y in 0..board_dimensions.1 {
					reel_symbols.push(game.get_slot(x, y).kind);
				}
				reel_symbols.push(game.board.rand_symbol());

				reels.push(Reel::new(reel_symbols));
			}

			UI {
				slot_size,
				margin,
				padding,
				win_dimensions,

				last_money: game.get_money(),
				
				game,
				reels,
				spinning: BoardSpinningState::Stopped,
				stopped_spinning: false,

				displaying_pay_lines: false,
				rl,
				thread,

				run_time: 0.0,
				run_time_loc,
				rainbow_shader,
				font
			}
		}

		pub fn step(&mut self) {
			let delta = self.rl.get_frame_time();
			self.run_time += delta;
			self.rainbow_shader.set_shader_value(self.run_time_loc, self.run_time);

			let mut all_reels_stopped = true;
			for x in 0..self.reels.len() {
				let mut reel_symbols = Vec::new();
				for y in 0..self.game.board.get_dimensions().1 {
					reel_symbols.push(self.game.get_slot(x, y).kind);
				}
				self.reels[x].step(delta, &reel_symbols, &mut self.game.board);

				if self.reels[x].spinning == ReelSpinningState::Spinning {
					all_reels_stopped = false;
				}
			}

			if all_reels_stopped && self.spinning != BoardSpinningState::Stopped {
				self.stopped_spinning = true;
				self.spinning = BoardSpinningState::Stopped;
			}
		}

		pub fn draw(&mut self) {
			let mut d = self.rl.begin_drawing(&self.thread);

			d.clear_background(Color::WHITE);
			d.draw_rectangle_rec(Rectangle {
				x: self.margin as f32,
				y: self.margin as f32,
				width: (self.win_dimensions.0 - self.margin*2) as f32,
				height: (self.win_dimensions.1 - self.margin*2 - 100) as f32
			}, Color::BLACK);

			for x in 0..self.reels.len() {
				let reel_pos = (
					self.margin + self.padding*(x as i32 + 1) + self.slot_size.x as i32*x as i32,
					self.margin+self.padding
				);
				
				self.reels[x].draw(Vector2 {
					x: reel_pos.0 as f32,
					y: reel_pos.1 as f32
				}, Vector2 {
					x: self.slot_size.x,
					y: self.slot_size.y*self.game.board.get_dimensions().1 as f32 + (self.padding as f32 * (self.game.board.get_dimensions().1 as f32 - 1.0))
				}, self.slot_size, self.padding as f32, &self.font, &self.rainbow_shader, &mut d)
			}

			if self.displaying_pay_lines {
				for line in &self.game.lines {
					d.draw_line_ex(
						Vector2 {
							x: (self.margin + self.padding*(line.0.0 as i32 + 1) + self.slot_size.x as i32*line.0.0 as i32 + self.slot_size.x as i32/2) as f32,
							y: (self.margin + self.padding*(line.0.1 as i32 + 1) + self.slot_size.y as i32*line.0.1 as i32 + self.slot_size.y as i32/2) as f32,
						},
						Vector2 {
							x: (self.margin + self.padding*(line.1.0 as i32 + 1) + self.slot_size.x as i32*line.1.0 as i32 + self.slot_size.x as i32/2) as f32,
							y: (self.margin + self.padding*(line.1.1 as i32 + 1) + self.slot_size.y as i32*line.1.1 as i32 + self.slot_size.y as i32/2) as f32,
						},
						5.0,
						Color::RED
					);
				}
			}
			d.draw_text(
				format!("Money: ${}",
					(self.last_money*100.0).floor()/100.0
				).as_str(),
				(self.margin as f32*1.5) as i32, self.win_dimensions.1 - 95, 20, Color::BLACK
			);
		}

		pub fn start_spinning(&mut self) {
			if self.spinning != BoardSpinningState::Stopped {
				return;
			}

			match self.game.spin() {
				Ok(cred) => {
					self.last_money = cred;
				}
				_ => ()
			};

			self.spinning = BoardSpinningState::Spinning;

			let mut i = 0;
			for reel in &mut self.reels {
				i += 1;

				reel.spinning = ReelSpinningState::Spinning;
				reel.scrolls_remaining = 10 + i*5;
			}
		}

		pub fn stop_spinning(&mut self) {
			if self.spinning != BoardSpinningState::Spinning {
				return;
			}
			
			let mut x = 0;
			for reel in &mut self.reels {
				// reel.spinning = ReelSpinningState::Returning;
				// let spring = &mut reel.reel_spring_group.springs[0];
				// 
				// spring.pos = 0.0;
				// spring.equilibrium = 0.0;
				// spring.vel = 10.0;

				// reel.reel_symbols.clear();
				// reel.reel_symbols.push(self.game.board.rand_symbol());
				// for y in 0..self.game.board.get_dimensions().1 {
					// reel.reel_symbols.push(self.game.get_slot(x, y).kind);
				// }
				// reel.reel_symbols.push(self.game.board.rand_symbol());
				
				self.spinning = BoardSpinningState::Stopping;
				reel.scrolls_remaining = x;
				x += 1;
			}
		}

		pub fn has_stopped_spinning(&mut self) -> bool {
			let stopped_spinning = self.stopped_spinning;
			self.stopped_spinning = false;
			return stopped_spinning;
		}
	}

	pub struct SpringGroup {
		pub springs: Vec<Spring>,

		pos_coefficients: (f32, f32), // pos, vel
		vel_coefficients: (f32, f32), // pos, vel
	}

	impl SpringGroup {
		fn new(springs: Option<Vec<Spring>>) -> SpringGroup {
			match springs {
				Some(springs) => {
					SpringGroup {
						springs,

						pos_coefficients: (0.0, 0.0),
						vel_coefficients: (0.0, 0.0)
					}
				},
				None => {
					SpringGroup {
						springs: Vec::new(),

						pos_coefficients: (0.0, 0.0),
						vel_coefficients: (0.0, 0.0)
					}
				}
			}
		}

		fn update(&mut self, delta: f32, angular_frequency: f32, damping_ratio: f32) {
			self.update_values(delta, angular_frequency, damping_ratio);
			self.update_springs();
		}

		fn update_values(&mut self, delta: f32, mut angular_frequency: f32, mut damping_ratio: f32) {
			if damping_ratio < 0.0 {
				damping_ratio = 0.0;
			}
			if angular_frequency < 0.0 {
				angular_frequency = 0.0;
			}

			// If spring is stationary, or isn't underdamped, return identity
			if angular_frequency < f32::EPSILON || damping_ratio >= 1.0 {
				self.pos_coefficients = (1.0, 0.0);
				self.vel_coefficients = (0.0, 1.0);
				return;
			}

			let omega_zeta = angular_frequency * damping_ratio;
			let alpha = angular_frequency * (1.0 - damping_ratio*damping_ratio).sqrt();

			let exp_term = (-omega_zeta * delta).exp();
			let cos_term = (alpha * delta).cos();
			let sin_term = (alpha * delta).sin();

			let inv_alpha = 1.0 / alpha;

			let exp_sin = exp_term*sin_term;
			let exp_cos = exp_term*cos_term;
			let exp_omega_zeta_sin_over_alpha = exp_term*omega_zeta*sin_term*inv_alpha;

			self.pos_coefficients = (
				exp_cos + exp_omega_zeta_sin_over_alpha,
				exp_sin*inv_alpha
			);

			self.vel_coefficients = (
				-exp_sin*alpha - omega_zeta*exp_omega_zeta_sin_over_alpha,
				exp_cos - exp_omega_zeta_sin_over_alpha
			);
		}

		fn update_springs(&mut self) {
			for spring in &mut self.springs {
				spring.update(&self.pos_coefficients, &self.vel_coefficients);
			}
		}
	}

	pub struct Spring {
		pos: f32,
		pub vel: f32,
		equilibrium: f32
	}

	impl Spring {
		fn update(&mut self, pos_coefficients: &(f32, f32), vel_coefficients: &(f32, f32)) {
			let old_pos = self.pos - self.equilibrium;
			let old_vel = self.vel;

			self.pos = old_pos*pos_coefficients.0 + old_vel*pos_coefficients.1 + self.equilibrium;
			self.vel = old_pos*vel_coefficients.0 + old_vel*vel_coefficients.1
		}
	}

	#[derive(PartialEq)]
	enum ReelSpinningState {
		Spinning,
		Returning,
		Stopped
	}

	pub struct Reel {
		pub reel_spring_group: SpringGroup,
		reel_symbols: Vec<SymbolType>,

		scrolls_remaining: i32,
		spinning: ReelSpinningState,
	}

	impl Reel {
		fn new(reel_symbols: Vec<SymbolType>) -> Reel {
			Reel {
				reel_spring_group: SpringGroup::new(Some(vec![Spring {
					pos: 0.0,
					vel: 0.0,
					equilibrium: 0.0
				}])),
				reel_symbols,

				scrolls_remaining: 0,
				spinning: ReelSpinningState::Stopped
			}
		}

		fn step(&mut self, delta: f32, symbol_board: &Vec<SymbolType>, board: &mut SlotBoard) {
			self.reel_spring_group.update(delta, 7.5, 0.6);

			let spring = &mut self.reel_spring_group.springs[0];
			if self.spinning == ReelSpinningState::Returning && spring.pos == 0.0 && spring.vel == 0.0 {
				self.spinning = ReelSpinningState::Stopped;
				return;
			}

			if self.spinning != ReelSpinningState::Spinning {
				return;
			}

			spring.equilibrium = 2.5;

			if spring.pos > 1.0 {
				spring.pos -= 1.0;
				if self.scrolls_remaining < 1 {
					if self.scrolls_remaining < -(symbol_board.len() as i32 - 1) {
						self.reel_symbols.remove(self.reel_symbols.len()-1);
						self.reel_symbols.insert(0, board.rand_symbol());
					} else {
						let idx = (symbol_board.len() as i32 - 1 as i32) as usize - (self.scrolls_remaining.abs() as usize);
						self.reel_symbols.remove(self.reel_symbols.len()-1);
						self.reel_symbols.insert(0, symbol_board[idx]);
					}
					if self.scrolls_remaining.abs() >= symbol_board.len() as i32 {
						spring.equilibrium = 0.0;
						self.spinning = ReelSpinningState::Returning;
					}
				} else {
					self.reel_symbols.remove(self.reel_symbols.len()-1);
					self.reel_symbols.insert(0, board.rand_symbol());
				}
				
				self.scrolls_remaining -= 1;
			}
		}

		fn draw(&self, pos: Vector2, size: Vector2, slot_size: Vector2, padding: f32, font: &Font, wild_shader: &Shader, d: &mut RaylibDrawHandle) {
			let mut d = d.begin_scissor_mode(pos.x as i32, pos.y as i32, size.x as i32, size.y as i32);

			let root_pos = Vector2 {
				x: pos.x,
				y: pos.y - padding - slot_size.y + slot_size.y*self.reel_spring_group.springs[0].pos
			};

			for i in 0..self.reel_symbols.len() {
				let symbol_kind = self.reel_symbols[i];

				let pos = (
					root_pos.x as i32,
					(root_pos.y + (padding as i32*i as i32) as f32 + slot_size.y*i as f32) as i32
				);
				
				d.draw_rectangle(
					pos.0,
					pos.1,
					slot_size.x as i32,
					slot_size.y as i32,
					Color::WHITE
				);
				
				let text_dim = get_text_size(symbol_kind.to_string(), &font, symbol_kind.font_size() as f32);
				
				if symbol_kind == SymbolType::Wild {
					{
						let mut d = d.begin_shader_mode(&wild_shader);
						d.draw_rectangle(pos.0, pos.1, slot_size.x as i32, slot_size.y as i32, Color::WHITE);
					}
				}
	
				d.draw_text_ex(
					&font,
					symbol_kind.to_string().as_str(),
					Vector2 {
						x: pos.0 as f32 + (slot_size.x/2.0 - text_dim.x/2.0),
						y: pos.1 as f32 + (slot_size.y/2.0 - text_dim.y/2.0)
					},
					symbol_kind.font_size() as f32,
					symbol_kind.font_size() as f32/10.0,
					if symbol_kind == SymbolType::Wild {Color::WHITE} else {Color::BLACK}
				);
			}
		}
	}
}