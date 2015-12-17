#![feature(core)]

extern crate core;

use std::collections::hash_map::HashMap;
use std::ffi::OsString;
use std::fs::{read_dir, Metadata, DirEntry};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::string::String;
use std::os::unix::raw;
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
	dev: Option<raw::dev_t>,
}

fn visit_dirs(ts: &mut ThreadState, top: &mut DirState, dir: &Path,
	process_file: &Fn(&mut DirState, &Metadata),
	merge_dir_state: &Fn(DirEntry, &mut DirState, DirState),
) {
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
			let mut ds = DirState::default();

			visit_dirs(ts, &mut ds, &entry.path(), process_file, merge_dir_state);
			merge_dir_state(entry, top, ds);

		} else if meta.is_file() {
			process_file(top, &meta);
		}
	}
}

fn tree<T: Display>(indent: &mut String, dir: &str, ds: DirState,
	get_value: &Fn(&DirState) -> T,
	test_cutoff: &Fn(&DirState) -> bool,
	sort: &Fn(&(OsString, DirState), &(OsString, DirState)) -> Ordering) {

	println!("{}{}  {}", indent, get_value(&ds), dir);

	let mut entries: Vec<(OsString, DirState)> = ds.directories.into_iter().collect();
	entries.sort_by(sort);

	let old_indent = indent.len();
	indent.push_str("    ");

	for entry in entries.into_iter() {
		if test_cutoff(&entry.1) {
			break;
		}
		tree(indent, &entry.0.to_string_lossy(), entry.1, get_value, test_cutoff, sort);
	}

	indent.truncate(old_indent);
}

fn main() {
	let options = args::parse_args();
	let mut ts = ThreadState::default();
	let mut root = DirState::default();

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

		visit_dirs(&mut ts, &mut root, Path::new(&path),
			&|ds, meta| {
				ds.total_size += meta.size() as u64;
				ds.blocks += meta.blocks() as u64;
				ds.number_of_files += 1;
			},
			&|entry, top, ds| {
				top.total_size += ds.total_size;
				top.blocks += ds.blocks;
				top.number_of_files += ds.number_of_files;
				top.directories.insert(entry.file_name(), ds);
			},
		);
	}
	let mut indent = String::new();

	match options.mode {
		args::Mode::ApparentSize => {
			let cutoff = (root.total_size as f64 * options.cutoff) as u64;
			tree(&mut indent, "", root,
				&|ds| numfmt::IecSizeShort(ds.total_size),
				&|ds| ds.total_size < cutoff,
				&|a, b| a.1.total_size.cmp(&b.1.total_size).reverse(),
			);
		},
		args::Mode::Size => {
			let cutoff = (root.blocks as f64 * options.cutoff) as u64;
			tree(&mut indent, "", root,
				&|ds| numfmt::IecSizeShort(ds.blocks * 512),
				&|ds| ds.blocks < cutoff,
				&|a, b| a.1.blocks.cmp(&b.1.blocks).reverse(),
			);
		},
		args::Mode::Files => {
			let cutoff = (root.number_of_files as f64 * options.cutoff) as u32;
			tree(&mut indent, "", root,
				&|ds| numfmt::SiFilesShort(ds.number_of_files as u64),
				&|ds| ds.number_of_files < cutoff,
				&|a, b| a.1.number_of_files.cmp(&b.1.number_of_files).reverse(),
			);
		},
	}
}
