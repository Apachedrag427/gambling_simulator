use raylib::prelude::*;

pub mod ui;
use ui::ui::UI;

fn main() {
	const SLOT_SIZE: Vector2 = Vector2 {
		x: 125.0,
		y: 125.0
	};
	const MARGIN: i32 = 5; // White pixels around the slot area
	const PADDING: i32 = 5; // Black pixels around the slots

	const BOARD_DIMENSIONS: (usize, usize) = (3, 3);

	let mut ui = UI::new(SLOT_SIZE, MARGIN, PADDING, BOARD_DIMENSIONS);


	while !ui.rl.window_should_close() {
		if let Some(key) = ui.rl.get_key_pressed() {
			if key == raylib::consts::KeyboardKey::KEY_SPACE {
				if !ui.displaying_pay_lines {
					match ui.game.spin() {
						_ => ()
					};
					if ui.game.lines.len() > 0 {
						ui.displaying_pay_lines = true;
					}
				} else {
					ui.displaying_pay_lines = false;
				}

			}
		}
		
		ui.draw();
	}
}
