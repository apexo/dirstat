extern crate argparse;

use self::argparse::{ArgumentParser, StoreTrue, Store, Collect};
use std::str::FromStr;

pub enum Mode {
	ApparentSize,
	Size,
	Files,
}

impl FromStr for Mode {
	type Err = ();

	fn from_str(src: &str) -> Result<Mode, ()> {
		return match src {
			"apparent-size" => Ok(Mode::ApparentSize),
			"size" => Ok(Mode::Size),
			"files" => Ok(Mode::Files),
			_ => Err(()),
		};
	}
}

pub struct Options {
	pub mode: Mode,
	pub cutoff: f64,
	pub xdev: bool,
	pub paths: Vec<String>,
}

pub fn parse_args() -> Options {
	let mut options = Options{
		mode: Mode::Size,
		cutoff: 0.003,
		xdev: true,
		paths: vec![".".to_string()],
	};

	{
		let mut parser = ArgumentParser::new();
		parser.set_description("Directory tree statistics.");

		parser.refer(&mut options.mode).add_option(&["-m", "--mode"], Store, "mode, one of: size, files, apparent-size; default: size");
		parser.refer(&mut options.cutoff).add_option(&["-c", "--cutoff"], Store, "cutoff; default: 0.003.");
		parser.refer(&mut options.xdev).add_option(&["--no-xdev"], StoreTrue, "enable cross-filesystem statistics; default: do not cross filesystem boundaries.");
		parser.refer(&mut options.paths).add_argument("path", Collect, "paths; default: .");

		parser.parse_args_or_exit();
	}

	options
}
