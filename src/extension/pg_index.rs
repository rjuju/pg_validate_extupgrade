use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	Index:indname:Index {
		indname: Name = ("indexrelid::regclass::text"),
		inddef: Text = ("pg_get_indexdef(indexrelid)"),
	}
}

impl Index {
	pub fn snapshot(client: &mut Transaction, relid: u32, pgver: u32)
		-> BTreeMap<String, Index>
	{
		let mut indexes = BTreeMap::new();

		let sql = format!("SELECT {} \
			FROM pg_index \
			WHERE indrelid = $1 \
			ORDER BY indexrelid::regclass::text",
			Index::tlist(pgver).join(", "),
		);

		let rows = client.query(&sql[..], &[&relid])
			.expect("Could net get pg_index rows");

		for row in &rows {
			let ind = Index::from_row(row);
			indexes.insert(ind.indname.clone(), ind);
		};

		indexes
	}
}
