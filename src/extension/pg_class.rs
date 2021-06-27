/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;
use postgres::{Row, Transaction};

use crate::{compare::*,
	CompareStruct, DbStruct,
	extension::pg_attribute::Attribute,
	pgtype::*,
};

DbStruct! {
	PgClass:relname:Relation {
		relname: Name,
		relkind: Char,
		relpersistence: Char,
	}
}

CompareStruct! {
	Relation {
		attributes: Vec<Attribute>,
		class: PgClass,
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
					rels.insert(r.ident.clone(), r);
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
		WHERE oid = $1",
		PgClass::tlist(pgver).join(", "),
	);

	let row = client.query_one(&sql[..], &[&oid])
		.expect("Could not get pg_class row");

	let class = PgClass::from_row(&row);

	let atts = Attribute::snapshot(client, oid, pgver);

	Some(
		Relation {
			ident: class.relname.clone(),
			attributes: atts,
			class,
		}
	)
}
