/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use crate::compare::{Compare};

// Can't have those function as default implementation as it's not possible to
// define extra Trait requirement for generic underlying types like Vec<T>
fn diff<T>(a: T, b: T, msg: &mut String)
	where T: std::cmp::PartialEq + std::fmt::Display
{
	if a != b {
		msg.push_str(&format!("\t- {}\n\t+ {}\n", a, b));
	}
}

fn value<T: std::fmt::Display>(item: T) -> String {
	format!("\n\t- {}\n", item)
}

// postgres crate defines postgres char as an i8, so we have to add a lot of
// useless code to output a char in the final diff as rust won't allow
// implement FromSql for rust char.  Of course, it also mean that we can't use
// i8 for anything else than postgres char.
pub type Char = i8;

impl Compare for Char {
	fn compare(&self, other: &Self, msg: &mut String) {
		let a: char = *self as u8 as char;
		let b: char = *other as u8 as char;
		diff(a, b, msg);
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

			impl Compare for $rust {
				fn compare(&self, other: &Self, msg: &mut String) {
					diff(self, other, msg);
				}

				fn value(&self) -> String {
					value(self)
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
	Name = String,
	Smallint = i16,
}