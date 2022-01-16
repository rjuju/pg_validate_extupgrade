/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2022 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	CompareStruct, DbStruct,
	elog::*,
	extension::pg_class::Relation,
	extension::pg_range::Range,
	pgdiff::SchemaDiff,
	pgtype::*,
	proc_prototype,
};

DbStruct! {
	PgType:typname:Type {
		typname: Text = ("t.oid::regtype::text"),
		typowner: Name = ("r.rolname"),
		typlen: Smallint,
		typbyval: Bool,
		typtype: Char,
		typcategory: Char,
		typispreferred: Bool,
		typisdefined: Bool,
		typdelim: Char,
		typrelid: Text = ("t.typrelid::regclass::text"),
		typsubscript: Text = (proc_prototype!("t.typsubscript")) {PG_14..},
		typelem: Text = ("t.typelem::regtype::text"),
		typarray: Text = ("t.typarray::regtype::text"),
		typinput: Text = (proc_prototype!("t.typinput")),
		typoutput: Text = (proc_prototype!("t.typoutput")),
		typreceive: Option<Text> = (proc_prototype!("t.typreceive")),
		typsend: Option<Text> = (proc_prototype!("t.typsend")),
		typmodin: Option<Text> = (proc_prototype!("t.typmodin")),
		typmodout: Option<Text> = (proc_prototype!("t.typmodout")),
		typanalyze: Option<Text> = (proc_prototype!("t.typanalyze")),
		typalign: Char,
		typstorage: Char,
		typnotnull: Bool,
		typndims: Integer,
		typcollation: Option<Name> = ("c.collname"),
		typdefault: Option<Text>,
		typacl: Option<Text> = ("t.typacl::text"),
		typenum: Option<Vec<Text>> = ("(SELECT array_agg(e.enumlabel || '=' || \
			e.enumsortorder) FROM pg_enum e WHERE enumtypid = t.oid)"),
	}
}

CompareStruct! {
	Type {
		typ: PgType,
		relation: Option<Relation>,
		range: Option<Range>,
	}
}

impl Type {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, Type>
	{
		let mut types = BTreeMap::new();

		for oid in oids {
			let typ = snap_one_type(client, oid, pgver);
			types.insert(typ.typ.typname.clone(), typ);
		}

		types
	}
}

pub fn snap_one_type(client: &mut Transaction, oid: u32, pgver: u32)
	-> Type
{
	let sql = format!("SELECT {}, typrelid as __typrelid \
		FROM pg_type t \
		JOIN pg_roles r ON r.oid = t.typowner \
		LEFT JOIN pg_collation c ON c.oid = t.typcollation \
		WHERE t.oid = $1",
		PgType::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	let typrelid = row.get("__typrelid");
	let typ = PgType::from_row(&row);

	// Raise a warning if a shell type is found, as that's probably not what
	// the extension author wants.
	if typ.typinput.starts_with("shell_in") ||
		typ.typoutput.starts_with("shell_out")
	{
		elog(WARNING,
			&format!("Shell type found for type {}", typ.typname));
	}

	let mut relation = None;
	if typrelid != 0 {
		let mut relations = Relation::snapshot(client, vec![typrelid], pgver);
		match relations.remove_entry(&typ.typname) {
			None => {
				elog(ERROR, &format!("Could not find relation for type {}",
						typ.typname));
				panic!();
			},
			Some((_, v)) => {
				relation = Some(v);
			}
		}
	}

	Type {
		ident: typ.typname.clone(),
		typ,
		relation: relation,
		range: Range::snapshot(client, oid, pgver),
	}
}
