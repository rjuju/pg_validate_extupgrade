use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	Trigger:tgname:Trigger {
		tgparentid: Name = ("tgparentid::regclass::text"),
		tgname: Name,
		tgdef: Text = ("pg_get_triggerdef(oid)"),
	}
}

impl Trigger {
	pub fn snapshot(client: &mut Transaction, relid: u32, pgver: u32)
		-> BTreeMap<String, Trigger>
	{
		let mut triggers = BTreeMap::new();

		let sql = format!("SELECT {} \
			FROM pg_trigger \
			WHERE NOT tgisinternal \
			AND tgrelid = $1",
			Trigger::tlist(pgver).join(", "),
		);

		let rows = client.query(&sql[..], &[&relid])
			.expect("Could net get pg_trigger rows");

		for row in &rows {
			let ind = Trigger::from_row(row);
			triggers.insert(ind.tgname.clone(), ind);
		};

		triggers
	}
}
