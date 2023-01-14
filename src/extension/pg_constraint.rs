/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2023 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	Constraint:conname:Constraint {
		conname: Name = ("nspname || '.' || conname"),
		condef: Text = ("pg_get_constraintdef(c.oid)"),
		comment: Option<Text> = ("obj_description(c.oid, 'pg_constraint')"),
	}
}

impl Constraint {
	pub fn snapshot_per_table(client: &mut Transaction, relid: u32, pgver: u32)
		-> BTreeMap<String, Constraint>
	{
		let mut cons = BTreeMap::new();

		let sql = format!("SELECT {} \
			FROM pg_constraint c \
			JOIN pg_namespace n ON n.oid = c.connamespace \
			WHERE conrelid = $1",
			Constraint::tlist(pgver).join(", "),
		);

		let rows = client.query(&sql[..], &[&relid])
			.expect("Could net get pg_constraint rows");

		for row in &rows {
			let con = Constraint::from_row(row);
			cons.insert(con.conname.clone(), con);
		};

		cons
	}
}

