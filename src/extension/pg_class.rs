/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;
use postgres::Transaction;

use crate::{compare::*,
	DbStruct,
	extension::pg_attribute::Attribute,
};

DbStruct! {
	Relation:relname {
		attributes: Vec<Attribute> {PG_NO_CATALOG},
		relname: String,
		relkind: String,
		relpersistence: String,
	}
}

impl Relation {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> HashMap<String, Relation>
	{
		let mut rels = HashMap::new();

		for oid in oids {
			match snap_one_class(client, oid, pgver) {
				Some(r) => {
					rels.insert(r.relname.clone(), r);
				},
				None => {}
			}
		}

		rels
	}
}

fn snap_one_class(client: &mut Transaction, oid: u32, pgver: u32)
	-> Option<Relation>
{
	let sql = format!("SELECT {} \
		FROM pg_class c \
		WHERE oid = {}",
		Relation::tlist(pgver).join(", "),
		oid);

	let rows = client.simple_query(&sql)
		.expect("Could not get pg_class row");

	if rows.len() != 2 {
		println!("Could not find pg_class entry for oid {}", oid);
		return None;
	}

	let rel = match &rows[0] {
		postgres::SimpleQueryMessage::Row(r) => r,
		_ => {
			println!("should not happen");
			std::process::exit(1);
		}
	};

	let relname = String::from(rel.get("relname").unwrap());
	let relkind = String::from(rel.get("relkind").unwrap());
	let relpersistence = String::from(rel.get("relpersistence").unwrap());

	let atts = Attribute::snapshot(client, oid, pgver);

	Some(
		Relation {
			relname,
			attributes: atts,
			relkind,
			relpersistence,
		}
	)
}
