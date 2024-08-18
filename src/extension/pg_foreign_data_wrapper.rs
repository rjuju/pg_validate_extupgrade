/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2024 : Julien Rouhaud - All rights reserved
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
	ForeignDataWrapper:fdwname:ForeignDataWrapper {
		fdwname: Text,
		fdwowner: Name = ("r.rolname"),
		fdwhandler: Option<Text> = (proc_prototype!("fdwhandler")),
		fdwvalidator: Option<Text> = (proc_prototype!("fdwvalidator")),
		fdwacl: Option<Text> = ("fdwacl::text"),
		fdwoptions: Option<ClassOptions>,
		comment: Option<Text> = ("d.description"),
	}
}

impl ForeignDataWrapper {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, ForeignDataWrapper>
	{
		let mut fdws = BTreeMap::new();

		for oid in oids {
			let fdw = snap_one_fdw(client, oid, pgver);
			fdws.insert(fdw.fdwname.clone(), fdw);
		}

		fdws
	}
}

pub fn snap_one_fdw(client: &mut Transaction, oid: u32, pgver: u32)
	-> ForeignDataWrapper
{
	let sql = format!("SELECT {} \
		FROM pg_foreign_data_wrapper fdw \
		JOIN pg_roles r ON r.oid = fdw.fdwowner \
		LEFT JOIN pg_catalog.pg_description d  ON d.classoid = fdw.tableoid \
			AND d.objoid = fdw.oid AND d.objsubid = 0 \
		WHERE fdw.oid = $1",
		ForeignDataWrapper::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	ForeignDataWrapper::from_row(&row)
}
