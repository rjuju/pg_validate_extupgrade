/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;
use diffy::create_patch;
use postgres::types::{FromSql, Type};

use crate::{
	compare::{Compare},
	pgdiff::{DiffSource, SchemaDiff},
};

// Can't have those function as default implementation as it's not possible to
// define extra Trait requirement for generic underlying types like Vec<T>
fn diff<'a, T>(a: T, b: T) -> Option<SchemaDiff<'a>>
	where T: std::cmp::PartialEq + std::fmt::Display
{
	if a != b {
		Some(SchemaDiff::Diff(a.to_string(), b.to_string()))
	} else {
		None
	}
}

fn value<T: std::fmt::Display>(item: T) -> String {
	format!("{}", item)
}

// postgres crate defines postgres char as an i8, so we have to add a lot of
// useless code to output a char in the final diff as rust won't allow
// implement FromSql for rust char.  Of course, it also mean that we can't use
// i8 for anything else than postgres char.
pub type Char = i8;

impl<'a> Compare<'a> for Char {
	fn compare(&self, other: &Self) -> Option<SchemaDiff<'a>> {
		let a: char = *self as u8 as char;
		let b: char = *other as u8 as char;
		diff(a, b)
	}

	fn value(&self) -> String {
		value(self)
	}
}

// HashMap implements PartialEq + Display and requires a specific
// implementation, so we can't use generic implementation for simple postgres
// type aliases.
macro_rules! PgAlias {
	{$($pg:ident = $rust:ident,)*} => {
		$(
			pub type $pg = $rust;

			impl<'a> Compare<'a> for $rust {
				fn compare(&self, other: &Self) -> Option<SchemaDiff<'a>> {
					diff(self, other)
				}

				fn value(&self) -> String {
					format!("{}", self)
				}
			}
		)*
	}
}

// Simple alias for rust/postgres translation. Reference:
// https://docs.rs/postgres-types/0.2.1/postgres_types/trait.FromSql.html
PgAlias!{
	Bool = bool,
	Integer = i32,
	Smallint = i16,
}

// Additional aliases for rust types having multiple corresponding postgres
// types, and some extra custom types
pub type Name = String;
pub type Text = String;

impl<'a> Compare<'a> for String {
	fn compare(&'a self, other: &'a Self) -> Option<SchemaDiff<'a>> {
		// If the content has at least one newline, use a diffy::Patch for
		// better readibility
		if self.matches('\n').count() > 0 {
			let patch = create_patch(&self, &other);

			// There's a mismatch if diffy returns at least one chunk
			match patch.hunks().len() {
				0 => { None },
				_ => { Some(SchemaDiff::UnifiedDiff(patch)) },
			}
		}
		else {
			// Otherwise return a simple Diff
			diff(self, other)
		}
	}

	fn value(&self) -> String {
		format!("{}", self)
	}
}

// Used for text[] column storing sets of key=value
#[derive(Debug)]
pub struct ClassOptions {
	options: HashMap<String, String>,
}

impl<'a> FromSql<'a> for ClassOptions {
	fn from_sql(ty: &Type, raw:&'a [u8])
		-> Result<ClassOptions, Box<dyn std::error::Error + Sync + Send>>
	{
		let mut options = HashMap::new();
		let vec = Vec::<String>::from_sql(ty, raw)?;

		for e in vec.iter() {
			let option: Vec<&str> = e.split("=").collect();

			if option.len() != 2 {
				panic!("Expected key=value format, found {}", e);
			}

			options.insert(String::from(option[0]), String::from(option[1]));
		}

		Ok(ClassOptions { options })
	}

	fn accepts(ty: &Type) -> bool {
		Vec::<String>::accepts(ty)
	}
}

impl<'a> Compare<'a> for ClassOptions {
	fn compare(&'a self, other: &'a Self) -> Option<SchemaDiff<'a>> {
		let mut missings:Vec<(DiffSource, Vec<&str>)> = Vec::new();
		let mut diffs = Vec::new();

		let mut missing_ins:Vec<&str> = vec![];
		for ident in other.options.keys() {
			if !self.options.contains_key(ident) {
				missing_ins.push(&ident[..]);
			}
		}
		if missing_ins.len() > 0 {
			missings.push((DiffSource::Installed, missing_ins));
		}

		let mut missing_upg:Vec<&str> = vec![];
		for ident in self.options.keys() {
			match other.options.get(ident) {
				None => {
					missing_upg.push(&ident[..]);
				},
				Some(o) => {
					if self.options.get(ident).unwrap() != o {
						diffs.push({
							SchemaDiff::NamedDiff(ident,
								self.options.get(ident).unwrap(),
								o,
							)
						})
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
				self.options.len(),
				other.options.len(),
				"Option",
				missings,
				diffs,
			))
		}
	}

	fn value(&self) -> String {
		self.options.iter()
			.map(|(k, v)| { format!("{}={}", k, v) })
			.collect::<Vec<String>>()
			.join(",")
	}
}
