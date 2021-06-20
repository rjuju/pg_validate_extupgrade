/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;

use postgres::Transaction;

mod pg_class;
use pg_class::Relation;

use crate::{compare::Compare, DbStruct};

mod pg_attribute;

DbStruct! {
	Extension {
		relations: Option<HashMap<String, Relation>>,
	}
}

impl Extension {
	pub fn snapshot(extname: &str, client: &mut Transaction) -> Self {
		let mut ext = Extension {
			ident: String::from(extname),
			relations: None,
		};

		client.execute("SET search_path TO pg_catalog", &[])
			.expect("Could not secure search_path");

		let dependencies = client.query(
			"SELECT classid::regclass::text, array_agg(objid) \
			FROM pg_depend d \
			JOIN pg_extension e ON e.oid = d.refobjid
			WHERE refclassid::regclass::text = 'pg_extension' \
			AND e.extname = $1 \
			GROUP BY 1", &[&extname]
		).expect("Could get the list of refclassid");

		for dependency in dependencies {
			let classid: &str = dependency.get(0);
			let objids: Vec<u32> = dependency.get(1);

			match classid {
				"pg_class" => {
					ext.relations = Some(Relation::snapshot(client, objids));
				}
				_ => {
					println!("Classid \"{}\" not handled", classid);
				}
			}
		}

		client.execute("RESET search_path", &[])
			.expect("Could not reset the search_path");

		//println!("{:#?}", ext);
		ext
	}
}
