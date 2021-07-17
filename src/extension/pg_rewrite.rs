use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	Rewrite:rulename:Rule {
		rulename: Name,
		ev_enabled: Char,
		inddef: Text = ("pg_get_ruledef(oid)"),
	}
}

impl Rewrite {
	pub fn snapshot(client: &mut Transaction, relid: u32, pgver: u32)
		-> BTreeMap<String, Rewrite>
	{
		let mut rewrites = BTreeMap::new();

		let sql = format!("SELECT {} \
			FROM pg_rewrite \
			WHERE ev_class = $1",
			Rewrite::tlist(pgver).join(", "),
		);

		let rows = client.query(&sql[..], &[&relid])
			.expect("Could net get pg_rewrite rows");

		for row in &rows {
			let ind = Rewrite::from_row(row);
			rewrites.insert(ind.rulename.clone(), ind);
		};

		rewrites
	}
}
