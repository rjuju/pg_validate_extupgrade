/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2025 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
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
	Range:rngtypid:Range {
		rngtypid: Text = ("r.rngtypid::regtype::text"),
		rngsubtype: Text = ("r.rngsubtype::regtype::text"),
		rngmultitypid: Text = ("r.rngmultitypid::regtype::text") {PG_14..},
		rngcollation: Option<Name> = ("c.collname"),
		rngsubopc: Name = ("opc.opcname"),
		rngcanonical: Option<Text> = (proc_prototype!("r.rngcanonical")),
		rngsubdiff: Option<Text> = (proc_prototype!("r.rngsubdiff")),
	}
}

impl Range {
	pub fn snapshot(client: &mut Transaction, oid: u32, pgver: u32)
		-> Option<Range>
	{
		let sql = format!("SELECT {} \
			FROM pg_range r \
			JOIN pg_opclass opc ON opc.oid = r.rngsubopc \
			LEFT JOIN pg_collation c ON c.oid = r.rngcollation \
			WHERE r.rngtypid = $1",
			Range::tlist(pgver).join(", "),
		);
	
		let row = match client.query_opt(&sql[..], &[&oid]) {
			Ok(r) => { r },
			Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
		};

		match row {
			Some(r) => { Some(Range::from_row(&r)) },
			None => { None },
		}
	}
}

