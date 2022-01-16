/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2022 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use postgres::{Transaction};

use crate::{pgtype::Guc};

impl Guc {
	pub fn snapshot(client: &mut Transaction, extver: String)
		-> Guc
	{
		let mut gucs = BTreeMap::new();
		let sql = "SELECT name, current_setting(name) AS value \
			FROM pg_settings";

		let rows = client.query(&sql[..], &[])
			.expect("Could not get pg_settings rows");

		for row in rows {
			gucs.insert(row.get("name"), row.get("value"));
		}

	Guc::new_from(extver, gucs)
	}
}
