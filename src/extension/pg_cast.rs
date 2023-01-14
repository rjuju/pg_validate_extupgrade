/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2023 : Julien Rouhaud - All rights reserved
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
	Cast:castname:Cast {
		castname: Text = ("castsource::regtype::text || ' -> ' || \
			casttarget::regtype::text"),
		castfunc: Option<Text> = (proc_prototype!("castfunc")),
		castcontext: Char,
		castmethod: Char,
		comment: Option<Text> = ("obj_description(oid, 'pg_cast')"),
	}
}

impl Cast {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, Cast>
	{
		let mut casts = BTreeMap::new();

		for oid in oids {
			let cast = snap_one_cast(client, oid, pgver);
			casts.insert(cast.castname.clone(), cast);
		}

		casts
	}
}

pub fn snap_one_cast(client: &mut Transaction, oid: u32, pgver: u32)
	-> Cast
{
	let sql = format!("SELECT {} \
		FROM pg_cast \
		WHERE oid = $1",
		Cast::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	Cast::from_row(&row)
}
