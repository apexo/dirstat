use std::fmt;

#[allow(dead_code)]
const IEC_LONG: &'static[&'static str] = &["kibi", "mebi", "gibi", "tebi", "pebi", "exbi"];
const IEC_SHORT: &'static[&'static str] = &["Ki", "Mi", "Gi", "Ti", "Pi", "Ei"];

#[allow(dead_code)]
const SI_LONG: &'static[&'static str] = &["kilo", "mega", "giga", "tera", "peta", "exa"];
const SI_SHORT: &'static[&'static str] = &["k", "M", "G", "T", "P", "E"];

fn iec_fmt(n: u64, f: &mut fmt::Formatter, prefixes: &[&str], singular: &str, plural: &str) -> fmt::Result {
	if n == 1 {
		write!(f, "{:5} {}", n, singular)
	} else if n < 1000 {
		write!(f, "{:5} {}", n, plural)
	} else {
		let mut index = 0;
		let mut base = 1024f64;

		while index < 5 && n as f64 >= base * 999.95 {
			index += 1;
			base *= 1024f64;
		}

		if (n as f64) < (base * 9.9995) {
			write!(f, "{:.3} {}{}", n as f64 / base, prefixes[index], plural)
		} else if (n as f64) < (base * 99.995) {
			write!(f, "{:.2} {}{}", n as f64 / base, prefixes[index], plural)
		} else {
			write!(f, "{:.1} {}{}", n as f64 / base, prefixes[index], plural)
		}
	}
}

fn si_fmt(n: u64, f: &mut fmt::Formatter, prefixes: &[&str], singular: &str, plural: &str) -> fmt::Result {
	if n == 1 {
		write!(f, "{:5} {}", n, singular)
	} else if n < 1000 {
		write!(f, "{:5} {}", n, plural)
	} else {
		let mut index = 0;
		let mut base = 1000f64;

		while index < 5 && n as f64 >= base * 999.95 {
			index += 1;
			base *= 1000f64;
		}

		if (n as f64) < (base * 9.9995) {
			write!(f, "{:.3} {}{}", n as f64 / base, prefixes[index], plural)
		} else if (n as f64) < (base * 99.995) {
			write!(f, "{:.2} {}{}", n as f64 / base, prefixes[index], plural)
		} else {
			write!(f, "{:.1} {}{}", n as f64 / base, prefixes[index], plural)
		}
	}
}

pub struct IecSizeShort(pub u64);
pub struct SiFilesShort(pub u64);

impl fmt::Display for IecSizeShort {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		iec_fmt(self.0, f, IEC_SHORT, "B", "B")
	}
}

impl fmt::Display for SiFilesShort {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		si_fmt(self.0, f, SI_SHORT, "file ", "files")
	}
}
