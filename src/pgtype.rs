/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2022 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use diffy::create_patch;
use postgres::types::{FromSql, Type};

use crate::{
	compare::{Compare, compare_map},
	pgdiff::SchemaDiff,
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

// BTreeMap implements PartialEq + Display and requires a specific
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
	Real = f32,
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
				_ => { Some(SchemaDiff::UnifiedDiff(None, patch)) },
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
	options: BTreeMap<String, String>,
}

impl ClassOptions {
	pub fn from_options(options: BTreeMap<String, String>) -> Self {
		ClassOptions { options }
	}
}

impl<'a> FromSql<'a> for ClassOptions {
	fn from_sql(ty: &Type, raw:&'a [u8])
		-> Result<ClassOptions, Box<dyn std::error::Error + Sync + Send>>
	{
		let mut options = BTreeMap::new();
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

fn class_options_cmp<'a>(ident: &'a str, self_option: &'a String,
	other_option: &'a String,
	diffs: &mut Vec<SchemaDiff<'a>>)
{
	if self_option != other_option {
		diffs.push({
			SchemaDiff::NamedDiff(ident,
				&self_option[..],
				&other_option[..],
			)
		});
	}
}

impl<'a> Compare<'a> for ClassOptions {
	fn compare(&'a self, other: &'a Self) -> Option<SchemaDiff<'a>>
	{
		compare_map(&self.options, &other.options, "Option",
			Some(class_options_cmp))
	}

	fn value(&self) -> String {
		self.options.iter()
			.map(|(k, v)| { format!("{}={}", k, v) })
			.collect::<Vec<String>>()
			.join(",")
	}
}

// Used for unordered array
#[derive(Debug)]
pub struct List {
	values: BTreeMap<String, ()>,
}

impl<'a> FromSql<'a> for List {
	fn from_sql(ty: &Type, raw:&'a [u8])
		-> Result<List, Box<dyn std::error::Error + Sync + Send>>
	{
		let mut values = BTreeMap::new();
		let vec = Vec::<String>::from_sql(ty, raw)?;

		for e in vec.iter() {
			values.insert(e.clone(), ());
		}

		Ok(List { values })
	}

	fn accepts(ty: &Type) -> bool {
		Vec::<String>::accepts(ty)
	}
}

impl<'a> Compare<'a> for List {
	fn compare(&'a self, other: &'a Self) -> Option<SchemaDiff<'a>>
	{
		compare_map(&self.values, &other.values, "Value", None)
	}
}

#[derive(Debug)]
pub struct ExecutedQueries {
	queries: BTreeMap<String, (usize, String)>,
}

impl ExecutedQueries {
	pub fn new() -> Self {
		ExecutedQueries { queries: BTreeMap::new() }
	}

	pub fn new_from(queries: BTreeMap<String, (usize, String)>) -> Self {
		ExecutedQueries { queries }
	}
}
fn query_cmp<'a>(query: &'a str, self_option: &'a (usize, String),
	other_option: &'a (usize, String),
	diffs: &mut Vec<SchemaDiff<'a>>)
{
	let patch = create_patch(&self_option.1, &other_option.1);

	match patch.hunks().len() {
		0 => { },
		_ => {
			let mut source = String::from(query);
			if self_option.0 != other_option.0 {
				source.push_str(&format!("\n-- {} rows\n++ {} rows\n",
						self_option.0, other_option.0));
			}
			diffs.push(SchemaDiff::UnifiedDiff(Some(source), patch));
		},
	}
}

impl<'a> Compare<'a> for ExecutedQueries {
	fn compare(&'a self, other: &'a Self) -> Option<SchemaDiff<'a>>
	{
		compare_map(&self.queries, &other.queries, "Resultset",
			Some(query_cmp))
	}
}

#[derive(Debug)]
pub struct Guc {
	extver: String,
	gucs: BTreeMap<String, String>,
}

impl<'a> Guc {
	pub fn new_from(extver: String, gucs: BTreeMap<String, String>) -> Self {
		Guc { extver, gucs }
	}
}

impl<'a> Compare<'a> for Guc {
	fn compare(&'a self, other: &'a Self) -> Option<SchemaDiff<'a>>
	{
		let mut diffs = Vec::new();

		// Extension might add new GUCs, so only warn about changing ones
		for (k, v) in self.gucs.iter() {
			if let Some(o) = other.gucs.get(k) {
				if v != o {
					diffs.push((&k[..], &o[..]));
				}
			}
		}

		match diffs.len() {
			0 => None,
			_ => Some(SchemaDiff::GucDiff(self.extver.clone(), diffs)),
		}
	}
}
