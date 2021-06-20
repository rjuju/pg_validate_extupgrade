/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use postgres::Transaction;

use crate::{compare::Compare, DbStruct};

DbStruct! {
	Attribute{
		attname: String,
	}
}

impl Attribute {
	pub fn snapshot(client: &mut Transaction, relid: u32) -> Vec<Attribute> {
		let mut atts = Vec::new();

		let sql = format!("SELECT * \
			FROM pg_attribute a \
			WHERE attnum > 0 \
			AND NOT attisdropped \
			AND attrelid = {} \
			ORDER BY attnum",
			relid,
		);

		let rows = client.simple_query(&sql)
			.expect("Could net get pg_attribute rows");

		for row in &rows {
			match row {
				postgres::SimpleQueryMessage::Row(r) => {
					atts.push(Attribute {
						ident: String::from(r.get("attname").unwrap()),
						attname: String::from(r.get("attname").unwrap()),
					})
				},
				postgres::SimpleQueryMessage::CommandComplete(n) => {
					assert!(*n == rows.len() as u64 - 1);
				},
				_ => { assert!(false) },
			};
		}

		atts
	}
}
