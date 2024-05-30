use phf::phf_map;

use weighted_rand::{builder::*, table::WalkerTable};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
	Nine,
	Ten,
	Jack,
	Queen,
	King,
	Ace,
	Coffee,
	Cake,
	Viking,
	Dragon,
	Wild
}

impl SymbolType {
	pub fn to_string(&self) -> String {
		match self {
			SymbolType::Nine => String::from("9"),
			SymbolType::Ten => String::from("10"),
			SymbolType::Jack => String::from("J"),
			SymbolType::Queen => String::from("Q"),
			SymbolType::King => String::from("K"),
			SymbolType::Ace => String::from("A"),
			SymbolType::Coffee => String::from("Coffee"),
			SymbolType::Cake => String::from("Cake"),
			SymbolType::Viking => String::from("Viking"),
			SymbolType::Dragon => String::from("Dragon"),
			SymbolType::Wild => String::from("Wild"),
		}
	}

	pub fn font_size(&self) -> i32 {
		match self {
			SymbolType::Nine => 70,
			SymbolType::Ten => 70,
			SymbolType::Jack => 70,
			SymbolType::Queen => 70,
			SymbolType::King => 70,
			SymbolType::Ace => 70,
			SymbolType::Coffee => 40,
			SymbolType::Cake => 40,
			SymbolType::Viking => 40,
			SymbolType::Dragon => 40,
			SymbolType::Wild => 60
		}
	}

	pub fn matches(&self, other: SymbolType) -> bool {
		*self == other || other == SymbolType::Wild
	}
}

#[derive(Debug)]
pub struct Symbol {
	pub kind: SymbolType,
	locked: bool
}


static TYPE_LIST: [SymbolType; 11] = [
	SymbolType::Nine,
	SymbolType::Ten,
	SymbolType::Jack,
	SymbolType::Queen,
	SymbolType::King,
	SymbolType::Ace,
	SymbolType::Coffee,
	SymbolType::Cake,
	SymbolType::Viking,
	SymbolType::Dragon,
	SymbolType::Wild
];
static TYPE_WEIGHTS: [f32; 11] = [
	30.0,
	30.0,
	20.0,
	15.0,
	10.0,
	8.0,
	7.0,
	5.0,
	3.0,
	1.0,
	0.75
];
static VALUE_LOOKUP: phf::Map<&str, i32> = phf_map! {
	"9" => 5,
	"10" => 7,
	"J" => 10,
	"Q" => 15,
	"K" => 20,
	"A" => 25,
	"Coffee" => 30,
	"Cake" => 40,
	"Viking" => 50,
	"Dragon" => 100,
	"Wild" => 125
};


#[derive(Debug)]
pub struct SlotBoard {
	width: usize,
	height: usize,

	slots: Vec<Vec<Symbol>>,

	wa_table: WalkerTable
}

impl SlotBoard {
	pub fn rand_symbol(&mut self) -> SymbolType {
		TYPE_LIST[self.wa_table.next_rng(&mut rand::thread_rng())]
	}

	fn new(width: usize, height: usize) -> SlotBoard {
		let builder = WalkerTableBuilder::new(&TYPE_WEIGHTS);
		let wa_table = builder.build();

		let mut rng = rand::thread_rng();

		let mut slots: Vec<Vec<Symbol>> = Vec::new();
		for _ in 0..width {
			let mut column: Vec<Symbol> = Vec::new();
			for _ in 0..height {
				column.push(Symbol {
					kind: TYPE_LIST[wa_table.next_rng(&mut rng)],
					locked: false
				});
			}
			slots.push(column);
		}
		SlotBoard {
			width,
			height,

			slots,
			wa_table
		}
	}

	fn spin(&mut self) {
		for x in 0..self.width {
			for y in 0..self.height {
				let symbol = &self.slots[x][y];
				if symbol.locked {
					continue;
				}

				self.slots[x][y] = Symbol {
					kind: self.rand_symbol(),
					locked: false
				};
			}
		}
	}

	pub fn get_dimensions(&self) -> (usize, usize) {
		(self.width, self.height)
	}
}

#[derive(Debug)]
struct PayLine {
	pub slots: Vec<(usize, usize)>
}

#[derive(Debug)]
enum PayLineResult {
	Success(SymbolType, Vec<((usize, usize), (usize, usize))>),
	Fail
}


impl PayLine {
	fn to_lines(&self) -> Vec<((usize, usize), (usize, usize))> {
		let mut lines = Vec::new();

		for i in 1..self.slots.len() {
			lines.push(((self.slots[i-1].0, self.slots[i-1].1), (self.slots[i].0, self.slots[i].1)));
		}

		lines
	}
	fn get_symbols(&self, board: &SlotBoard) -> Vec<SymbolType> {
		let mut symbols = Vec::new();
		for (x, y) in &self.slots {
			symbols.push(board.slots[*x][*y].kind);
		}
		symbols
	}

	fn check(&self, board: &SlotBoard) -> PayLineResult {
		let symbols = self.get_symbols(board);
		let mut stored_type = symbols[0];
		for i in 1..symbols.len() {
			if stored_type == SymbolType::Wild {
				stored_type = symbols[i];
			}
			if symbols[i] != stored_type && symbols[i] != SymbolType::Wild {
				return PayLineResult::Fail;
			}
		}
		PayLineResult::Success(stored_type, self.to_lines())
	}
}

#[derive(Debug)]
pub struct SlotGame {
	pub board: SlotBoard,

	credits: f64,
	// denom is in cents
	denom: i32,
	// bet is in credits
	bet: i32,

	pub lines: Vec<((usize, usize), (usize, usize))>,
	paylines: Vec<PayLine>
}

#[derive(Debug)]
pub enum SpinError {
	InsufficientCredits
}

impl SlotGame {
	pub fn new(width: usize, height: usize) -> SlotGame {
		let paylines: Vec<PayLine> = vec![

			// Vertical
			PayLine {
				slots: vec![
					(0, 0),
					(0, 1),
					(0, 2)
				]
			},
			PayLine {
				slots: vec![
					(1, 0),
					(1, 1),
					(1, 2)
				]
			},
			PayLine {
				slots: vec![
					(2, 0),
					(2, 1),
					(2, 2)
				]
			},

			// Horizontal
			PayLine {
				slots: vec![
					(0, 0),
					(1, 0),
					(2, 0)
				]
			},
			PayLine {
				slots: vec![
					(0, 1),
					(1, 1),
					(2, 1)
				]
			},
			PayLine {
				slots: vec![
					(0, 2),
					(1, 2),
					(2, 2)
				]
			},

			// Top Left -> Bottom Right
			PayLine {
				slots: vec![
					(0, 0),
					(1, 1),
					(2, 2)
				]
			},

			// Bottom Left -> Top Right
			PayLine {
				slots: vec![
					(0, 2),
					(1, 1),
					(2, 0)
				]
			},
		];

		SlotGame {
			board: SlotBoard::new(width, height),
			
			denom: 1,
			bet: 50,
			credits: 100_00.0,
			lines: Vec::new(),
			paylines
		}
	}

	pub fn get_denom(&self) -> i32 {
		self.denom
	}
	pub fn set_denom(&mut self, n: i32) {
		self.credits *= self.denom as f64 / n as f64;
		self.denom = n;
	}

	pub fn get_bet(&self) -> i32 {
		self.bet
	}
	pub fn set_bet(&mut self, n: i32) {
		self.bet = n;
	}

	pub fn get_credits(&self) -> f64 {
		self.credits
	}

	pub fn get_money(&self) -> f64 {
		self.credits / (self.denom as f64 * 100.0)
	}

	fn pay_symbol(&mut self, kind: SymbolType) {
		let val = VALUE_LOOKUP[kind.to_string().as_str()];
		self.credits += val as f64 * (self.bet / 3) as f64;
	}

	fn pay(&mut self) {
		self.lines.clear();

		let mut results = Vec::new();
		for payline in &self.paylines {
			results.push(payline.check(&self.board));
		}

		for result in results {
			match result {
				PayLineResult::Success(kind, lines) => {
					self.pay_symbol(kind);
					self.lines.extend(lines);
				},
				PayLineResult::Fail => ()
			};
		}
	}
	pub fn spin(&mut self) -> Result<f64, SpinError> {
		if (self.credits - self.bet as f64) < 0.0 {
			return Err(SpinError::InsufficientCredits);
		}
		self.credits -= self.bet as f64;
		let money = self.get_money();
		self.board.spin();
		self.pay();
		println!("\nCredits: {}", self.credits);
		println!("Money: {}\n", self.get_credits() as f32 / (self.get_denom() as f32 * 100_f32));
		Ok(money)
	}

	pub fn get_slot(&self, x: usize, y: usize) -> &Symbol {
		return &self.board.slots[x][y];
	}
}