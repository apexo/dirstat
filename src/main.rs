#![feature(path_ext)]
#![feature(core)]

extern crate core;

use std::collections::hash_map::HashMap;
use std::ffi::OsString;
use std::fs::{read_dir, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::string::String;
use std::os::unix::raw;
use std::fs::PathExt;
use std::fmt::Display;
use core::cmp::Ordering;

mod numfmt;
mod args;

#[derive(Default)]
struct DirState {
	number_of_files: u32,
	total_size: u64,
	blocks: u64,
	directories: HashMap<OsString, DirState>,
}

#[derive(Default)]
struct ThreadState {
	root: DirState,
	dirstack: Vec<DirState>,
	dev: Option<raw::dev_t>,
}

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

		if let Some(dev) = ts.dev {
			if dev != meta.dev() {
				continue;
			}
		}

		if meta.is_dir() {
			ts.dirstack.push(Default::default());

			visit_dirs(ts, &entry.path(), cb);

			let ds = ts.dirstack.pop().unwrap();

			let top = ts.dirstack.last_mut().unwrap_or(&mut ts.root);
			top.total_size += ds.total_size;
			top.blocks += ds.blocks;
			top.number_of_files += ds.number_of_files;
			top.directories.insert(entry.file_name(), ds);
		} else if meta.is_file() {
			cb(ts, &meta);
		}
	}
}

fn tree<T: Display>(indent: &mut String, dir: &str, ds: DirState,
	get_value: &Fn(&DirState) -> T,
	test_cutoff: &Fn(&DirState) -> bool,
	sort: &Fn(&(OsString, DirState), &(OsString, DirState)) -> Ordering) {

	println!("{}{}  {}", indent, get_value(&ds), dir);

	let mut entries: Vec<(OsString, DirState)> = Vec::new();
	entries.extend(ds.directories.into_iter());
	entries.sort_by(sort);

	for _ in 0..4 {
		indent.push(' ');
	}

	for entry in entries.into_iter() {
		if test_cutoff(&entry.1) {
			break;
		}
		tree(indent, &entry.0.to_string_lossy(), entry.1, get_value, test_cutoff, sort);
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
			let ds = ts.dirstack.last_mut().unwrap_or(&mut ts.root);
			ds.total_size += meta.size() as u64;
			ds.blocks += meta.blocks() as u64;
			ds.number_of_files += 1;
		});
	}
	let mut indent = String::new();

	match options.mode {
		args::Mode::ApparentSize => {
			let cutoff = (ts.root.total_size as f64 * options.cutoff) as u64;
			tree(&mut indent, "", ts.root,
				&|ds| numfmt::IecSizeShort(ds.total_size),
				&|ds| ds.total_size < cutoff,
				&|a, b| a.1.total_size.cmp(&b.1.total_size).reverse(),
			);
		},
		args::Mode::Size => {
			let cutoff = (ts.root.blocks as f64 * options.cutoff) as u64;
			tree(&mut indent, "", ts.root,
				&|ds| numfmt::IecSizeShort(ds.blocks * 512),
				&|ds| ds.blocks < cutoff,
				&|a, b| a.1.blocks.cmp(&b.1.blocks).reverse(),
			);
		},
		args::Mode::Files => {
			let cutoff = (ts.root.number_of_files as f64 * options.cutoff) as u32;
			tree(&mut indent, "", ts.root,
				&|ds| numfmt::SiFilesShort(ds.number_of_files as u64),
				&|ds| ds.number_of_files < cutoff,
				&|a, b| a.1.number_of_files.cmp(&b.1.number_of_files).reverse(),
			);
		},
	}
}
