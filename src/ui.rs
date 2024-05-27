pub mod ui {
    use raylib::prelude::*;

	use gambling_simulator::{SlotGame, SymbolType};

	pub struct UI {
		slot_size: Vector2,
		win_dimensions: (i32, i32),
		margin: i32,
		padding: i32,

		pub game: SlotGame,

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
			let game = SlotGame::new(board_dimensions.0, board_dimensions.1);

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

			UI {
				slot_size,
				margin,
				padding,
				win_dimensions,
				
				game,

				displaying_pay_lines: false,
				rl,
				thread,

				run_time: 0.0,
				run_time_loc,
				rainbow_shader,
				font
			}
		}

		pub fn draw(&mut self) {
			self.run_time += self.rl.get_frame_time();
			self.rainbow_shader.set_shader_value(self.run_time_loc, self.run_time);
			let mut d = self.rl.begin_drawing(&self.thread);

			d.clear_background(Color::WHITE);
			d.draw_rectangle_rec(Rectangle {
				x: self.margin as f32,
				y: self.margin as f32,
				width: (self.win_dimensions.0 - self.margin*2) as f32,
				height: (self.win_dimensions.1 - self.margin*2 - 100) as f32
			}, Color::BLACK);

			for x in 0..self.game.board.get_dimensions().0 as i32 {
				for y in 0..self.game.board.get_dimensions().1 as i32 {
					let pos = (
						self.margin + self.padding*(x + 1) + self.slot_size.x as i32*x,
						self.margin + self.padding*(y + 1) + self.slot_size.y as i32*y
					);
					d.draw_rectangle(
						pos.0,
						pos.1,
						self.slot_size.x as i32,
						self.slot_size.y as i32,
						Color::WHITE
					);
					let symbol_kind = self.game.get_slot(x as usize, y as usize).kind;
					let text_dim = get_text_size(symbol_kind.to_string(), &self.font, symbol_kind.font_size() as f32);
					if symbol_kind == SymbolType::Wild {
						{
							let mut d = d.begin_shader_mode(&self.rainbow_shader);
							d.draw_rectangle(pos.0, pos.1, self.slot_size.x as i32, self.slot_size.y as i32, Color::WHITE);
						}
					}

					d.draw_text_ex(
						&self.font,
						symbol_kind.to_string().as_str(),
						Vector2 {
							x: pos.0 as f32 + (self.slot_size.x as i32/2 - text_dim.x as i32/2) as f32,
							y: pos.1 as f32 + (self.slot_size.y as i32/2 - text_dim.y as i32/2) as f32
						},
						symbol_kind.font_size() as f32,
						symbol_kind.font_size() as f32/10.0,
						if symbol_kind == SymbolType::Wild {Color::WHITE} else {Color::BLACK}
					);
				}
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
					self.game.get_credits() as f32 / (self.game.get_denom() as f32 * 100_f32)
				).as_str(),
				(self.margin as f32*1.5) as i32, self.win_dimensions.1 - 95, 20, Color::BLACK
			);
		}
	}
}