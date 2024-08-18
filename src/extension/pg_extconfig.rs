/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2024 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{
	compare::*,
	DbStruct,
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	ExtConfig:extname:ExtConfig {
		extname: Name,
		options: ClassOptions,
	}
}

impl ExtConfig {
	pub fn snapshot(client: &mut Transaction, extname: &str)
		-> Self
	{
		let sql = format!("SELECT unnest(extconfig)::regclass::text AS config, \
					unnest(extcondition) AS condition \
				FROM pg_extension \
				WHERE extname = $1",
		);

		let rows = client.query(&sql[..], &[&extname])
			.expect("Could net get pg_extension rows");

		let mut options: BTreeMap<String, String> = BTreeMap::new();
		for row in rows {
			options.insert(row.try_get("config").unwrap(),
			row.try_get("condition").unwrap());
		}

		ExtConfig {
			extname: extname.to_string(),
			options: ClassOptions::from_options(options),
		}
	}
}
