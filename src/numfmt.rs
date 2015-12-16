use std::fmt;

#[allow(dead_code)]
const IEC_LONG: &'static[&'static str] = &["kibi", "mebi", "gibi", "tebi", "pebi", "exbi"];
const IEC_SHORT: &'static[&'static str] = &["Ki", "Mi", "Gi", "Ti", "Pi", "Ei"];
const IEC_BASE: f64 = 1024f64;

#[allow(dead_code)]
const SI_LONG: &'static[&'static str] = &["kilo", "mega", "giga", "tera", "peta", "exa"];
const SI_SHORT: &'static[&'static str] = &["k", "M", "G", "T", "P", "E"];
const SI_BASE: f64 = 1000f64;

fn num_fmt(n: u64, f: &mut fmt::Formatter, base: f64, prefixes: &[&str], singular: &str, plural: &str) -> fmt::Result {
	if n == 1 {
		write!(f, "{:5} {}", n, singular)
	} else if n < 1000 {
		write!(f, "{:5} {}", n, plural)
	} else {
		let mut index = 0;
		let mut unit = base;

		while index < 5 && n as f64 >= unit * 999.95 {
			index += 1;
			unit *= base;
		}

		if (n as f64) < (unit * 9.9995) {
			write!(f, "{:.3} {}{}", n as f64 / unit, prefixes[index], plural)
		} else if (n as f64) < (unit * 99.995) {
			write!(f, "{:.2} {}{}", n as f64 / unit, prefixes[index], plural)
		} else {
			write!(f, "{:.1} {}{}", n as f64 / unit, prefixes[index], plural)
		}
	}
}

pub struct IecSizeShort(pub u64);
pub struct SiFilesShort(pub u64);

impl fmt::Display for IecSizeShort {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		num_fmt(self.0, f, IEC_BASE, IEC_SHORT, "B", "B")
	}
}

impl fmt::Display for SiFilesShort {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		num_fmt(self.0, f, SI_BASE, SI_SHORT, "file ", "files")
	}
}
