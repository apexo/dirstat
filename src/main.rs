#![feature(path_ext)]

use std::collections::hash_map::HashMap;
use std::ffi::OsString;
use std::fs::{read_dir, Metadata};
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::string::String;
use std::os::unix::raw;
use std::fs::PathExt;

mod numfmt;
mod args;

struct DirState {
	number_of_files: u32,
	total_size: u64,
	blocks: u64,
	directories: HashMap<OsString, DirState>,
}

impl Default for DirState {
	fn default() -> DirState {
		DirState{
			number_of_files: 0,
			total_size: 0,
			blocks: 0,
			directories: HashMap::new(),
		}
	}
}

impl Add<DirState> for DirState {
	type Output = DirState;

	fn add(mut self, rhs: DirState) -> DirState {
		let mut result = DirState::default();
		result.number_of_files = self.number_of_files + rhs.number_of_files;
		result.total_size = self.total_size + rhs.total_size;
		result.blocks = self.blocks + rhs.blocks;

		for (dir, state) in rhs.directories.into_iter() {
			match self.directories.remove(&dir) {
				None => { result.directories.insert(dir, state); },
				Some(state0) => { result.directories.insert(dir, state0 + state); },
			}
		}
		for (dir, state) in self.directories.into_iter() {
			result.directories.insert(dir, state);
		}
		result
	}
}


struct ThreadState {
	dirstack: Vec<DirState>,
	dev: Option<raw::dev_t>,
}

impl Default for ThreadState {
	fn default() -> ThreadState {
		ThreadState{
			dirstack: vec![DirState::default()],
			dev: None,
		}
	}
}

impl ThreadState {
	fn root_ref(&self) -> &DirState {
		self.dirstack.first().unwrap()
	}

	fn root(self) -> DirState {
		assert!(self.dirstack.len() == 1);
		self.dirstack.into_iter().next().unwrap()
		//self.dirstack.pop().unwrap()
	}
}

/*
impl Add<ThreadState> for ThreadState {
	type Output = ThreadState;

	fn add(self, rhs: ThreadState) -> ThreadState {
		ThreadState{
			number_of_files: self.number_of_files + rhs.number_of_files,
			total_size: self.total_size + rhs.total_size,
			blocks: self.blocks + rhs.blocks,
		}
	}
}
*/

fn visit_dirs(ts: &mut ThreadState, dir: &Path, cb: &mut FnMut(&mut ThreadState, &Metadata)) {
	let iter = match read_dir(dir) {
		Ok(iter) => iter,
		Err(error) => {
			println!("read_dir error: {} @ {:?}", error, dir);
			return;
		}
	};

	for entry in iter {
		let entry = match entry {
			Ok(entry) => entry,
			Err(error) => {
				println!("readdir error: {} @ {:?}", error, dir);
				continue;
			}
		};

		let meta = match entry.metadata() {
			Ok(meta) => meta,
			Err(error) => {
				println!("metadata error: {} @ {:?}", error, dir);
				continue;
			}
		};

		match ts.dev {
			Some(dev) => { 
				if dev != meta.dev() {
					continue;
				}
			}
			None => {},
		}

		if meta.is_dir() {
			ts.dirstack.push(Default::default());

			visit_dirs(ts, &entry.path(), cb);

			let ds = ts.dirstack.pop().unwrap();

			{
				let top = ts.dirstack.last_mut().unwrap();
				top.total_size += ds.total_size;
				top.blocks += ds.blocks;
				top.number_of_files += ds.number_of_files;
			}

			ts.dirstack.last_mut().unwrap().directories.insert(entry.file_name(), ds);

		} else if meta.is_file() {
			cb(ts, &meta);
		}
	}
}

fn tree_size(indent: &mut String, dir: &str, cutoff_size: u64, ds: DirState) {
	println!("{}{}  {}", indent, numfmt::IecSizeShort(ds.total_size), dir);

	let mut entries: Vec<(OsString, DirState)> = Vec::new();
	entries.extend(ds.directories.into_iter());
	entries.sort_by(|a, b| a.1.total_size.cmp(&b.1.total_size).reverse());

	for _ in 0..4 {
		indent.push(' ');
	}

	for entry in entries.into_iter() {
		if entry.1.total_size < cutoff_size {
			break;
		}
		tree_size(indent, &entry.0.to_string_lossy(), cutoff_size, entry.1);
	}

	for _ in 0..4 {
		indent.pop();
	}
}

fn tree_blocks(indent: &mut String, dir: &str, cutoff_blocks: u64, ds: DirState) {
	println!("{}{}  {}", indent, numfmt::IecSizeShort(ds.blocks * 512), dir);

	let mut entries: Vec<(OsString, DirState)> = Vec::new();
	entries.extend(ds.directories.into_iter());
	entries.sort_by(|a, b| a.1.blocks.cmp(&b.1.blocks).reverse());

	for _ in 0..4 {
		indent.push(' ');
	}

	for entry in entries.into_iter() {
		if entry.1.blocks < cutoff_blocks {
			break;
		}
		tree_blocks(indent, &entry.0.to_string_lossy(), cutoff_blocks, entry.1);
	}

	for _ in 0..4 {
		indent.pop();
	}
}

fn tree_files(indent: &mut String, dir: &str, cutoff_files: u32, ds: DirState) {
	println!("{}{}  {}", indent, numfmt::SiFilesShort(ds.number_of_files as u64), dir);

	let mut entries: Vec<(OsString, DirState)> = Vec::new();
	entries.extend(ds.directories.into_iter());
	entries.sort_by(|a, b| a.1.blocks.cmp(&b.1.blocks).reverse());

	for _ in 0..4 {
		indent.push(' ');
	}

	for entry in entries.into_iter() {
		if entry.1.number_of_files < cutoff_files {
			break;
		}
		tree_files(indent, &entry.0.to_string_lossy(), cutoff_files, entry.1);
	}

	for _ in 0..4 {
		indent.pop();
	}
}

fn main() {
	let options = args::parse_args();
	let mut ts = ThreadState::default();

	for path in options.paths {
		ts.dev = if options.xdev {
			match Path::new(&path).metadata() {
				Ok(meta) => Some(meta.dev()),
				Err(error) => {
					println!("metadata error: {} @ {:?}", error, path);
					continue;
				}
			}
		} else {
			None
		};

		visit_dirs(&mut ts, Path::new(&path), &mut |ts, meta| {
			let ds = ts.dirstack.last_mut().unwrap();
			ds.total_size += meta.size() as u64;
			ds.blocks += meta.blocks() as u64;
			ds.number_of_files += 1;
		});
	}
	//println!("{} ({} allocated) in {}", numfmt::IecSizeShort(ts.root().total_size), numfmt::IecSizeShort(ts.root().blocks * 512), numfmt::SiFilesShort(ts.root().number_of_files as u64));

	let mut indent = String::new();

	match options.mode {
		args::Mode::ApparentSize => tree_size(&mut indent, "", (ts.root_ref().total_size as f64 * options.cutoff) as u64, ts.root()),
		args::Mode::Size => tree_blocks(&mut indent, "", (ts.root_ref().blocks as f64 * options.cutoff) as u64, ts.root()),
		args::Mode::Files => tree_files(&mut indent, "", (ts.root_ref().number_of_files as f64 * options.cutoff) as u32, ts.root()),
	}
}
