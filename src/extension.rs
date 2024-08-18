/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2024 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;

use postgres::Transaction;

mod pg_cast;
use pg_cast::Cast;
mod pg_class;
use pg_class::Relation;
mod pg_event_trigger;
use pg_event_trigger::EventTrigger;
mod pg_extconfig;
use pg_extconfig::ExtConfig;
mod pg_foreign_data_wrapper;
use pg_foreign_data_wrapper::ForeignDataWrapper;
mod pg_namespace;
use pg_namespace::Namespace;
mod pg_opclass;
use pg_opclass::OpClass;
mod pg_opfamily;
use pg_opfamily::OpFamily;
mod pg_operator;
use pg_operator::Operator;
mod pg_proc;
use pg_proc::Routine;
mod pg_type;
use pg_type::Type;

use crate::{
	compare::*,
	CompareStruct,
	pgdiff::SchemaDiff,
	pgtype::ExecutedQueries,
};

mod pg_aggregate;
mod pg_attribute;
mod pg_constraint;
mod pg_index;
mod pg_policy;
mod pg_range;
mod pg_rewrite;
mod pg_trigger;
mod pg_statistic_ext;

CompareStruct! {
	Extension {
		relations: Option<BTreeMap<String, Relation>>,
		extension_config: ExtConfig,
		routines: Option<BTreeMap<String, Routine>>,
		event_triggers: Option<BTreeMap<String, EventTrigger>>,
		operators: Option<BTreeMap<String, Operator>>,
		types: Option<BTreeMap<String, Type>>,
		casts: Option<BTreeMap<String, Cast>>,
		foreign_data_wrappers: Option<BTreeMap<String, ForeignDataWrapper>>,
		namespaces: Option<BTreeMap<String, Namespace>>,
		opclasses: Option<BTreeMap<String, OpClass>>,
		opfamilies: Option<BTreeMap<String, OpFamily>>,
		extra_queries: ExecutedQueries,
	}
}

impl Extension {
	pub fn snapshot(extname: &str, client: &mut Transaction, pgver: u32)
		-> Self
	{
		let extension_config = ExtConfig::snapshot(client, extname);

		let mut ext = Extension {
			ident: String::from(extname),
			relations: None,
			extension_config,
			routines: None,
			event_triggers: None,
			operators: None,
			types: None,
			casts: None,
			foreign_data_wrappers: None,
			namespaces: None,
			opclasses: None,
			opfamilies: None,
			extra_queries: ExecutedQueries::new(),
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
				"pg_cast" => {
					ext.casts = Some(Cast::snapshot(client,
							objids, pgver));
				},
				"pg_class" => {
					ext.relations = Some(Relation::snapshot(client,
							objids, pgver));
				},
				"pg_event_trigger" => {
					assert!(pgver >= PG_9_3,
						"Event triggers were introduced in PostgreSQL 9.3");
					ext.event_triggers = Some(EventTrigger::snapshot(client,
							objids, pgver));
				},
				"pg_foreign_data_wrapper" => {
					ext.foreign_data_wrappers = Some(
						ForeignDataWrapper::snapshot(client, objids, pgver));
				},
				"pg_namespace" => {
					ext.namespaces = Some(Namespace::snapshot(client,
							objids, pgver));
				},
				"pg_opclass" => {
					ext.opclasses = Some(OpClass::snapshot(client,
							objids, pgver));
				},
				"pg_opfamily" => {
					ext.opfamilies = Some(OpFamily::snapshot(client,
							objids, pgver));
				},
				"pg_operator" => {
					ext.operators = Some(Operator::snapshot(client,
							objids, pgver));
				},
				"pg_proc" => {
					ext.routines = Some(Routine::snapshot(client,
							objids, pgver));
				},
				"pg_type" => {
					ext.types = Some(Type::snapshot(client,
							objids, pgver));
				},
				_ => {
					println!("Classid \"{}\" not handled", classid);
				},
			}
		}

		client.execute("RESET search_path", &[])
			.expect("Could not reset the search_path");

		ext
	}

	pub fn set_extra_queries(&mut self, extra_queries: ExecutedQueries) {
		self.extra_queries = extra_queries;
	}
}
