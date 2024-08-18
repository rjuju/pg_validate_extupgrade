/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2024 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	elog::*,
	pgdiff::SchemaDiff,
	pgtype::*,
	opr_prototype,
	proc_prototype,
};

DbStruct! {
	Aggregate:aggname:Aggregate {
		aggname: Name = (proc_prototype!("a.aggfnoid")),
		aggkind: Char {PG_9_4..},
		aggnumdirectargs: Smallint {PG_9_4..},
		aggtransfn: Text = (proc_prototype!("a.aggtransfn")),
		aggfinalfn: Option<Text> = (proc_prototype!("a.aggfinalfn")),
		aggcombinefn: Option<Text> = (proc_prototype!("a.aggcombinefn")) {PG_9_6..},
		aggserialfn: Option<Text> = (proc_prototype!("a.aggserialfn")) {PG_9_6..},
		aggdeserialfn: Option<Text> = (proc_prototype!("a.aggdeserialfn")) {PG_9_6..},
		aggmtransfn: Option<Text> = (proc_prototype!("a.aggmtransfn")) {PG_9_4..},
		aggminvtransfn: Option<Text> = (proc_prototype!("a.aggminvtransfn")) {PG_9_4..},
		aggmfinalfn: Option<Text> = (proc_prototype!("a.aggmfinalfn")) {PG_9_4..},
		aggfinalextra: Bool {PG_9_4..},
		aggmfinalextra: Bool {PG_9_4..},
		aggfinalmodify: Char {PG_11..},
		aggmfinalmodify: Char {PG_11..},
		aggsortop: Option<Text> = (opr_prototype!("o")),
		aggtranstype: Text = ("aggtranstype::regtype::text"),
		aggtransspace: Integer {PG_9_4..},
		aggmtranstype: Option<Text> = ("aggmtranstype::regtype::text") {PG_9_4..},
		aggmtransspace: Integer {PG_9_4..},
		agginitval: Option<Text>,
		aggminitval: Option<Text> {PG_9_4..},
		comment: Option<Text> = ("obj_description(a.aggfnoid, 'pg_aggregate')"),
	}
}

impl Aggregate {
	pub fn snap_one_aggregate(client: &mut Transaction, oid: u32, pgver: u32)
		-> Option<Aggregate>
	{
		let sql = format!("SELECT {} \
			FROM pg_aggregate a \
			LEFT JOIN pg_operator o ON o.oid = a.aggsortop \
			WHERE a.aggfnoid = $1",
			Aggregate::tlist(pgver).join(", "),
		);
	
		let row = match client.query_opt(&sql[..], &[&oid]) {
			Ok(r) => { r },
			Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
		};

		match row {
			None => { None },
			Some(r) => { Some(Aggregate::from_row(&r)) },
		}
	}
}
