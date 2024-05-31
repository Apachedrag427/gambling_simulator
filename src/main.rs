use raylib::prelude::*;

pub mod ui;
use ui::ui::{BoardSpinningState, UI};

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
		ui.step();

		if let Some(key) = ui.rl.get_key_pressed() {
			if key == raylib::consts::KeyboardKey::KEY_SPACE {
				if !ui.displaying_pay_lines && ui.spinning == BoardSpinningState::Stopped {
					ui.start_spinning();
				} else if ui.spinning == BoardSpinningState::Spinning {
					ui.stop_spinning();
				} else if ui.spinning != BoardSpinningState::Stopping {
					ui.displaying_pay_lines = false;
				}
			}
		}

		if ui.has_stopped_spinning() {
			ui.last_money = ui.game.get_money();
			if ui.game.lines.len() > 0 {
				ui.displaying_pay_lines = true;
			}
		}
		
		ui.draw();
	}
}
