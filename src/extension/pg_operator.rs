/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	elog::*,
	pgdiff::SchemaDiff,
	pgtype::*,
	proc_prototype,
};

DbStruct! {
	Operator:oprname:Operator {
		oprname: Name = ("o.oid::regoper::text || \
			'(' || oprleft::regtype::text \
			|| ',' || \
			oprright::regtype::text || ')'"),
		oprowner: Name = ("r.rolname"),
		oprkind: Char,
		oprcanmerge: Bool,
		oprcanhash: Bool,
		oprleft: Option<Name> = ("o.oprleft::regtype::text"),
		oprright: Name = ("o.oprright::regtype::text"),
		// may be zero for shell operator, allow NULL here and raise a warning
		// later if needed
		oprresult: Option<Name> = ("o.oprresult::regtype::text"),
		oprcom: Option<Name> = ("o.oprcom::regtype::text"),
		oprnegate: Option<Name> = ("o.oprnegate::regtype::text"),
		// may be zero for shell operator, allow NULL here and raise a warning
		// later if needed
		oprcode: Option<Text> = (proc_prototype!("o.oprcode")),
		oprrest: Option<Text> = (proc_prototype!("o.oprrest")),
		oprjoin: Option<Text> = (proc_prototype!("o.oprjoin")),
	}
}

impl Operator {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, Operator>
	{
		let mut operators = BTreeMap::new();

		for oid in oids {
			let operator = snap_one_operator(client, oid, pgver);
			operators.insert(operator.oprname.clone(), operator);
		}

		operators
	}
}

pub fn snap_one_operator(client: &mut Transaction, oid: u32, pgver: u32)
	-> Operator
{
	let sql = format!("SELECT {} \
		FROM pg_operator o \
		JOIN pg_roles r ON r.oid = o.oprowner \
		WHERE o.oid = $1",
		Operator::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	let operator = Operator::from_row(&row);

	// Raise a warning if a shell type is found, as that's probably not what
	// the extension author wants.
	if operator.oprresult.is_none() || operator.oprcode.is_none() {
		elog(WARNING,
			&format!("Shell type found for operator {}", operator.oprname));
	}

	operator
}
