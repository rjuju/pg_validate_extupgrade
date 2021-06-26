/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
};

DbStruct! {
	Attribute:attname:Attribute {
		attname: String,
	}
}

impl Attribute {
	pub fn snapshot(client: &mut Transaction, relid: u32, pgver: u32)
		-> Vec<Attribute>
	{
		let mut atts = Vec::new();

		let sql = format!("SELECT {} \
			FROM pg_attribute a \
			WHERE attnum > 0 \
			AND NOT attisdropped \
			AND attrelid = $1 \
			ORDER BY attnum",
			Attribute::tlist(pgver).join(", "),
		);

		let rows = client.query(&sql[..], &[&relid])
			.expect("Could net get pg_attribute rows");

		for row in &rows {
			atts.push(Attribute::from_row(row));
		};

		atts
	}
}
