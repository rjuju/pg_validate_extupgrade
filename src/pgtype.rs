/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use crate::compare::{Compare, diff};

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
}

impl Compare for String {
	fn compare(&self, other: &String, msg: &mut String) {
		diff(self, other, msg);
	}
}
