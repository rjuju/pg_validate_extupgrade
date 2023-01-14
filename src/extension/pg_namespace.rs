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
};

DbStruct! {
	Namespace:nspname:Namespace {
		nspname: Text,
		nspowner: Name = ("r.rolname"),
		nspacl: Option<Text> = ("nspacl::text"),
		comment: Option<Text> = ("obj_description(nsp.oid, 'pg_namespace')"),
	}
}

impl Namespace {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, Namespace>
	{
		let mut nsps = BTreeMap::new();

		for oid in oids {
			let nsp = snap_one_nsp(client, oid, pgver);
			nsps.insert(nsp.nspname.clone(), nsp);
		}

		nsps
	}
}

pub fn snap_one_nsp(client: &mut Transaction, oid: u32, pgver: u32)
	-> Namespace
{
	let sql = format!("SELECT {} \
		FROM pg_namespace nsp \
		JOIN pg_roles r ON r.oid = nsp.nspowner \
		WHERE nsp.oid = $1",
		Namespace::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	Namespace::from_row(&row)
}
