use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	CompareStruct, DbStruct,
	elog::*,
	extension::pg_aggregate::Aggregate,
	pgdiff::SchemaDiff,
	pgtype::*,
	proc_prototype,
};

DbStruct! {
	PgRoutine:signature:Routine {
		signature: Text = (proc_prototype!("p.oid")),
		proowner: Name = ("r.rolname"),
		prolang: Name = ("l.lanname"),
		procost: Real,
		prorows: Real,
		prosupport: Text = (proc_prototype!("p.prosupport")) {PG_12..},
		prokind: Char {PG_11..},
		prosecdef: Bool,
		proleakproof: Bool,
		proisstrict: Bool,
		provolatile: Char,
		proparallel: Char {PG_9_6..},
		prorettype: Option<Text> = ("pg_get_function_result(p.oid)"),
		prosrc: Text,
		prosqlbody: Text = ("pg_get_function_sqlbody(p.oid)") {PG_14..},
		proconfig: Option<ClassOptions>,
		proacl: Option<Text> = ("proacl::text"),
		comment: Option<Text> = ("obj_description(p.oid, 'pg_proc')"),
		probin: Option<Text>,
	}
}

CompareStruct! {
	Routine {
		routine: PgRoutine,
		aggregate: Option<Aggregate>,
	}
}

impl Routine {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, Routine>
	{
		let mut routines = BTreeMap::new();

		for oid in oids {
			let r = snap_one_routine(client, oid, pgver);
			routines.insert(r.routine.signature.clone(), r);
		}

		routines
	}
}

pub fn snap_one_routine(client: &mut Transaction, oid: u32, pgver: u32)
	-> Routine
{
	let sql = format!("SELECT {} \
		FROM pg_proc p \
		JOIN pg_roles r on r.oid = p.proowner \
		JOIN pg_language l on l.oid = p.prolang \
		WHERE p.oid = $1",
		PgRoutine::tlist(pgver).join(", "),
	);

	let row = match client.query_one(&sql[..], &[&oid]) {
		Ok(r) => { r },
		Err(e) => { elog(ERROR, &format!("{}", e)); panic!(); },
	};

	let pgroutine = PgRoutine::from_row(&row);
	let aggregate = Aggregate::snap_one_aggregate(client, oid, pgver);

	Routine {
		ident: pgroutine.signature.clone(),
		routine: pgroutine,
		aggregate,
	}
}
