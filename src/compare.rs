/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;
use postgres::row::Row;

pub const PG_9_3: u32 = 90300;
pub const PG_9_4: u32 = 90400;
pub const PG_10: u32 = 100000;
pub const PG_12: u32 = 120000;
pub const PG_13: u32 = 130000;
pub const PG_14: u32 = 140000;

pub const PG_MIN: u32 = 0;
pub const PG_MAX: u32 = u32::MAX;

pub trait Compare {
	fn compare(&self, other: &Self, msg: &mut String);
	fn typname() -> &'static str {
		panic!("Should not be called.");
	}
	fn value(&self) -> String {
		format!("one")
	}
}

pub trait Sql {
	fn tlist(server_version_num: u32) -> Vec<String>;
	fn from_row(row: &Row) -> Self;
}

impl<T: Compare> Compare for Vec<T>
where T: std::fmt::Debug
{
	fn compare(&self, other: &Vec<T>, msg: &mut String) {

		for (i,(a,b)) in self.iter().zip(other.iter()).enumerate() {
			let mut res = String::new();
			a.compare(b, &mut res);
			if res != "" {
				msg.push_str(&format!("elem {}:\n{}", i, res));
			}
		}
	}

	fn typname() -> &'static str {
		<T>::typname()
	}

	fn value(&self) -> String {
		format!("\n\t- {:?}\n", self)
	}
}

impl<T: Compare> Compare for Option<T> {
	fn compare(&self, other: &Option<T>, msg: &mut String) {
		if self.is_none() && !other.is_none() {
			let res = format!("installed has no value, while upgraded \
				has {}", other.value());
			msg.push_str(&res);
			return;
		}

		if !self.is_none() && other.is_none() {
			let res = format!("upgraded has no value, while installed \
				has {}", self.as_ref().unwrap().value());
			msg.push_str(&res);
			return;
		}

		if self.is_none() && other.is_none() {
			return;
		}

		let src = self.as_ref().unwrap();
		let dst = other.as_ref().unwrap();

		src.compare(dst, msg);
	}

	fn typname() -> &'static str {
		<T>::typname()
	}
}

impl<T: Compare> Compare for HashMap<String, T> {
	fn compare(&self, other: &HashMap<String, T>, msg: &mut String) {
		if self.len() < other.len() {
			let mut res = format!("Upgraded version has {} more {t} ({}) than \
				installed version ({})\nMissing {t}:\n",
				other.len() - self.len(),
				other.len(),
				self.len(),
				t = <T>::typname(),
			);

			for ident in other.keys() {
				if !self.contains_key(ident) {
					res.push_str(&format!("\t- {}\n", ident));
				}
			}

			msg.push_str(&res);
			return;
		}

		if self.len() > other.len() {
			let mut res = format!("Installed version has {} more {t} ({}) than \
				upgraded version ({})\nMissing {t}:\n",
				self.len() - other.len(),
				self.len(),
				other.len(),
				t = <T>::typname(),
			);

			for ident in self.keys() {
				if !other.contains_key(ident) {
					res.push_str(&format!("\t- {}\n", ident));
				}
			}

			msg.push_str(&res);
			return;
		}


		let mut missing = HashMap::new();
		let mut res = String::new();
		let mut tmp = String::new();

		// Find missing or different objects in upgraded version
		for (n, r) in self {
			let other = other.get(n);

			if other.is_none() {
				tmp.push_str(&format!("\t- {}\n", n));
			} else {
				let other = other.unwrap();
				r.compare(other, &mut res);
			}
		}
		missing.insert("installed", tmp);

		// Find missing objects in installed version.  Different objects are
		// already checked.
		tmp = String::new();
		for (n, _) in other {
			let src = self.get(n);

			if src.is_none() {
				tmp.push_str(&format!("\t- {}\n", n));
			}
		}
		missing.insert("upgraded", tmp);

		for (k, v) in missing {
			if v != "" {
				msg.push_str(&format!("Missing {} in {} version:\n",
						<T>::typname(), k));
				msg.push_str(&v);
				msg.push('\n');
			}
		}

		if res != "" {
			msg.push_str(&res);
		}
	}

	fn typname() -> &'static str {
		<T>::typname()
	}
}

#[macro_export]
macro_rules! CompareStruct {
	($struct:ident {$( $field:ident:$type:ty ),*,}) => {
		#[derive(Debug)]
		pub struct $struct {
			ident: String,
			$($field: $type),*
		}

		impl Compare for $struct {
			fn compare(&self, other: &Self, msg: &mut String) {
				$(
					let mut res = String::new();
					self.$field.compare(&other.$field, &mut res);
					if res != "" {
						// If this is an inner structure holding some catalog
						// data, display the original error to avoid an extra
						// indirection message.
						if stringify!($type).starts_with("Pg") {
							msg.push_str(&res);
						} else {
							msg.push_str(&format!(
									"Mismatch found for {} {} in {}:\n{}\n",
									&stringify!($struct).to_string(),
									&self.ident,
									&stringify!($field).to_string(),
									&res,
							));
						}
					}
				)*
			}

			fn typname() -> &'static str {
				stringify!($struct)
			}
		}
	};
}

#[macro_export]
macro_rules! DbStruct {
	// This part of the marco transforms the given T in Option<T> if it depends
	// on a postgres major version.
	(field $min:tt:$max:tt:$t:ty:$f:ident) => {
		Option<$t>
	};

	(field $minormax:tt:$t:ty:$f:ident) => {
		Option<$t>
	};

	(field $t:ty:$f:ident) => {
		$t
	};

	// This part of the macro generate the final struct
	($struct:ident:$ident:ident:$typname:ident {
		$(
			$field:ident:$type:ty $(=($expr:literal))?
			$( { $($pgmin:ident)? .. $($pgmax:ident)? } )?
		),*,
	}) => {
		#[derive(Debug)]
		pub struct $struct {
			$(
				$field:
				DbStruct!(
					// optional minimum pg major version (inclusive) and
					// maximum pg major version (exclusive)
					field $($($pgmin :)?)?  $($($pgmax :)?)? $type:$field
				)
			),*
		}

		impl Compare for $struct {
			fn compare(&self, other: &Self, msg: &mut String) {
				$(
					let mut res = String::new();
					self.$field.compare(&other.$field, &mut res);
					if res != "" {
						msg.push_str(&format!(
								"Mismatch found for {} {} in {}:\n{}\n",
								&stringify!($typname).to_string(),
								&self.$ident,
								&stringify!($field).to_string(),
								&res,
						));
					}
				)*
			}

			fn typname() -> &'static str {
				stringify!($typname)
			}
		}

		impl Sql for $struct {
			fn tlist(server_version_num: u32) -> Vec<String> {
				let mut tlist = vec![];
				$(
					let _pgmin = PG_MIN;
					$($(let _pgmin = $pgmin;)?)?
					let _pgmax = PG_MAX;
					$($(let _pgmax = $pgmax;)?)?

					if server_version_num >= _pgmin as u32 &&
						server_version_num < _pgmax
					{
						let _expr = stringify!($field);
						let _as = String::from("");
						$(
							let _expr = $expr;
							let _as = format!(" AS {}", stringify!($field));
						)?

						tlist.push(format!("{}{}",
								_expr.to_string(),
								_as,
						));
					} else {
						let mut typname = stringify!($type).to_lowercase();
						if typname == "char" {
							typname = String::from("\"char\"");
						}
						tlist.push(format!(
								"NULL::{} AS {}",
								typname,
								stringify!($field),
							)
						);
					}
				)*
				tlist
			}

			fn from_row(row: &Row) -> Self {
				$struct {
					$(
						$field: row.get(stringify!($field)),
					)*
				}
			}
		}
	};
}
