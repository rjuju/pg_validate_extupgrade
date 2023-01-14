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
	OpClass:opcname:OpClass {
		opcname: Text = ("n.nspname || '.' || opc.opcname"),
		opcmethod: Name = ("am.amname"),
		opcowner: Name = ("r.rolname"),
		opcfamily: Name = ("opfn.nspname || '.' || opf.opfname"),
		opcintype: Name = ("opc.opcintype::regtype::text"),
		opcdefault: Bool,
		opckeytype: Option<Name> = ("opc.opckeytype::regtype::text"),
	}
}

impl OpClass {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, OpClass>
	{
		let mut opcs = BTreeMap::new();

		for oid in oids {
			let opc = snap_one_opc(client, oid, pgver);
			opcs.insert(opc.opcname.clone(), opc);
		}

		opcs
	}
}

pub fn snap_one_opc(client: &mut Transaction, oid: u32, pgver: u32)
	-> OpClass
{
	let sql = format!("SELECT {} \
		FROM pg_opclass opc \
		JOIN pg_namespace n ON n.oid = opc.opcnamespace \
		JOIN pg_am am ON am.oid = opc.opcmethod \
		JOIN pg_opfamily opf ON opf.oid = opc.opcfamily \
		JOIN pg_namespace opfn ON opfn.oid = opf.opfnamespace \
		JOIN pg_roles r ON r.oid = opc.opcowner \
		WHERE opc.oid = $1",
		OpClass::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	OpClass::from_row(&row)
}
