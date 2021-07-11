/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;
use postgres::row::Row;
use crate::pgdiff::{SchemaDiff, DiffSource};

pub const PG_9_3: u32 = 90300;
pub const PG_9_4: u32 = 90400;
pub const PG_10: u32 = 100000;
pub const PG_12: u32 = 120000;
pub const PG_13: u32 = 130000;
pub const PG_14: u32 = 140000;

pub const PG_MIN: u32 = 0;
pub const PG_MAX: u32 = u32::MAX;

pub trait Compare {
	fn compare(&self, other: &Self) -> Option<SchemaDiff>;
	fn typname() -> &'static str {
		panic!("Should not be called.");
	}
	fn value(&self) -> String {
		panic!("Should not be called");
	}
}

pub trait Sql {
	fn tlist(server_version_num: u32) -> Vec<String>;
	fn from_row(row: &Row) -> Self;
}

impl<T: Compare> Compare for Vec<T>
where T: std::fmt::Debug
{
	fn compare(&self, other: &Vec<T>) -> Option<SchemaDiff> {
		let mut ms = vec![];

		// First check all elements in the "self" array
		for (i, a) in self.iter().enumerate() {
			match other.get(i) {
				None => {
					ms.push((i, Box::new(SchemaDiff::NoneDiff(
									DiffSource::Upgraded,
									self.get(i).unwrap().value()),
					)));
				},
				Some(v) => {
					if let Some(m) = a.compare(v) {
						ms.push((i, Box::new(m)));
					}
				}
			}
		}

		// And check extraneous element in the "other" array, if if has more
		// elements
		for i in self.len()..other.len() {
			ms.push((self.len() - 1 + i, Box::new(SchemaDiff::NoneDiff(
							DiffSource::Installed,
							other.get(i).unwrap().value(),
							),
			)));
		}

		match ms.len() {
			0 => { None },
			_ => { Some(SchemaDiff::VecDiff(
					self.len(),
					other.len(),
					//<T>::typname(),
					ms)) },
		}
	}

	fn typname() -> &'static str {
		<T>::typname()
	}

	fn value(&self) -> String {
		format!("{:?}", self)
	}
}

impl<T: Compare> Compare for Option<T> {
	fn compare(&self, other: &Option<T>) -> Option<SchemaDiff> {
		if self.is_none() && !other.is_none() {
			return Some(
				SchemaDiff::NoneDiff(
					DiffSource::Installed,
					other.value(),
				)
			);
		}

		if !self.is_none() && other.is_none() {
			return Some(
				SchemaDiff::NoneDiff(
					DiffSource::Upgraded,
					self.value(),
				));
		}

		if self.is_none() && other.is_none() {
			return None;
		}

		let src = self.as_ref().unwrap();
		let dst = other.as_ref().unwrap();

		src.compare(dst)
	}

	fn typname() -> &'static str {
		<T>::typname()
	}

	fn value(&self) -> String {
		match self {
			None => {
				assert!(false, "Should not be called");
				panic!();
			},
			Some(v) => { v.value() },
		}
	}
}

impl<T: Compare> Compare for HashMap<String, T> {
	fn compare(&self, other: &HashMap<String, T>) -> Option<SchemaDiff> {
		let mut missings = Vec::new();
		let mut diffs = Vec::new();

		let mut missing_ins = vec![];
		for ident in other.keys() {
			if !self.contains_key(ident) {
				missing_ins.push(ident.clone());
			}
		}
		if missing_ins.len() > 0 {
			missings.push((DiffSource::Installed, missing_ins));
		}

		let mut missing_upg = vec![];
		for ident in self.keys() {
			match other.get(ident) {
				None => {
					missing_upg.push(ident.clone());
				},
				Some(o) => {
					match self.get(ident).unwrap().compare(o) {
						None => {},
						Some(d) => {
							diffs.push(d);
						},
					}
				}
			}
		}
		if missing_upg.len() > 0 {
			missings.push((DiffSource::Upgraded, missing_upg));
		}

		if missings.len() == 0 && diffs.len() == 0 {
			None
		} else {
			Some(SchemaDiff::HashMapDiff(
				self.len(),
				other.len(),
				<T>::typname(),
				missings,
				diffs,
			))
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
			fn compare(&self, other: &Self) -> Option<SchemaDiff> {
				let mut vec = vec![];

				$(
					if let Some(m) = self.$field.compare(&other.$field) {
						let f;

						// If this is an inner structure holding some catalog
						// data, display the original error to avoid an extra
						// indirection message.
						if stringify!($type).starts_with("Pg") {
							match m {
								SchemaDiff::StructDiff(_,_,mut v) => {
									vec.append(&mut v);
								},
								_ => {
									panic!("Expected a StructDiff, found:\n \
										{:#?}", m);
								}
							};
						} else {
							f = Some(stringify!($field).to_string());
							vec.push((f, m));
						}
					}
				)*

				match vec.len() {
					0 => None,
					_ => Some(SchemaDiff::StructDiff(
							stringify!($struct).to_string(),
							self.ident.clone(),
							vec,
					))
				}
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
			fn compare(&self, other: &Self) -> Option<SchemaDiff> {
				let mut vec = vec![];

				$(
					if let Some(m) = self.$field.compare(&other.$field) {
						vec.push((Some(stringify!($field).to_string()), m));
					}
				)*

				match vec.len() {
					0 => None,
					_ => Some(SchemaDiff::StructDiff(
							stringify!($typname).to_string(),
							self.$ident.clone(),
							vec,
					))
				}
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
