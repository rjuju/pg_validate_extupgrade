use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	Policy:polname:Policy {
		polname: Name,
		polcmd: Char,
		polpermissive: Bool {PG_10..},
		polroles: List = ("CASE \
			WHEN p.polroles = '{0}'::oid[] \
				THEN string_to_array('public'::text, ''::text)::name[]
			ELSE ARRAY( SELECT pg_authid.rolname
			   FROM pg_authid \
			  WHERE pg_authid.oid = ANY (p.polroles) \
			  ORDER BY pg_authid.rolname) \
			END"),
		polqual: Option<Text> = ("pg_get_expr(p.polqual, p.polrelid)"),
		polwithcheck: Option<Text> = ("pg_get_expr(p.polwithcheck, p.polrelid)"),
		comment: Option<Text> = ("obj_description(oid, 'pg_policy')"),
	}
}

impl Policy {
	pub fn snapshot(client: &mut Transaction, relid: u32, pgver: u32)
		-> BTreeMap<String, Policy>
	{
		let mut policies = BTreeMap::new();

		let sql = format!("SELECT {} \
			FROM pg_policy p \
			WHERE polrelid = $1",
			Policy::tlist(pgver).join(", "),
		);

		let rows = client.query(&sql[..], &[&relid])
			.expect("Could net get pg_policy rows");

		for row in &rows {
			let pol = Policy::from_row(row);
			policies.insert(pol.polname.clone(), pol);
		};

		policies
	}
}
