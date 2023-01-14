/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2023 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::BTreeMap;
use postgres::{Row, Transaction};

use crate::{compare::*,
	CompareStruct, DbStruct,
	extension::pg_attribute::Attribute,
	extension::pg_constraint::Constraint,
	extension::pg_index::Index,
	extension::pg_policy::Policy,
	extension::pg_rewrite::Rewrite,
	extension::pg_trigger::Trigger,
	extension::pg_statistic_ext::{ExtendedStatistic,
		PG_MIN_VER as EXT_STATS_MIN_VER},
	pgdiff::SchemaDiff,
	pgtype::*,
};

DbStruct! {
	PgClass:relname:Relation {
		relname: Text = ("c.oid::regclass::text"),
		reloftype: Text = ("reloftype::regtype::text"),
		relowner: Name = ("r.rolname"),
		relam: Option<Name> = ("am.amname"),
		relhasindex: Bool,
		relpersistence: Char,
		relkind: Char,
		relchecks: Smallint,
		relhasrules: Bool,
		relhastriggers: Bool,
		relrowsecurity: Bool {PG_9_4..},
		relforcerowsecurity: Bool {PG_9_4..},
		relispopulated: Bool {PG_9_3..},
		relreplident: Char {PG_9_4..},
		relispartition: Bool {PG_10..},
		relpartkey: Text = ("pg_get_partkeydef(c.oid)") {PG_10..},
		relacl: Option<Text> = ("relacl::text"),
		reloptions: Option<ClassOptions>,
		relpartbound: Text = ("pg_get_expr(c.relpartbound, c.oid)") {PG_10..},
		comment: Option<Text> = ("obj_description(c.oid, 'pg_class')"),
	}
}

CompareStruct! {
	Relation {
		attributes: Vec<Attribute>,
		indexes: BTreeMap<String, Index>,
		stats: Option<BTreeMap<String, ExtendedStatistic>>,
		constraints: BTreeMap<String, Constraint>,
		rules: BTreeMap<String, Rewrite>,
		triggers: BTreeMap<String, Trigger>,
		policies: BTreeMap<String, Policy>,
		class: PgClass,
	}
}

impl Relation {
	pub fn snapshot<'a>(client: &mut Transaction, oids: Vec<u32>, pgver: u32)
		-> BTreeMap<String, Relation>
	{
		let mut rels = BTreeMap::new();

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
		JOIN pg_roles r ON r.oid = c.relowner \
		LEFT JOIN pg_am am ON am.oid = c.relam \
		WHERE c.oid = $1",
		PgClass::tlist(pgver).join(", "),
	);

	let row = client.query_one(&sql[..], &[&oid])
		.expect("Could not get pg_class row");

	let class = PgClass::from_row(&row);

	// Index should not be seen as direct dependency of an extension and will
	// be handled explicitly
	assert!(!class.relkind != 'i' as Char);

	let atts = Attribute::snapshot(client, oid, pgver);
	let indexes = Index::snapshot(client, oid, pgver);

	let stats = match pgver {
		EXT_STATS_MIN_VER..=PG_MAX => {
			Some(ExtendedStatistic::snapshot(client, oid, pgver))
		},
		_ => None,
	};

	let constraints = Constraint::snapshot_per_table(client, oid, pgver);
	let rules = Rewrite::snapshot(client, oid, pgver);
	let triggers = Trigger::snapshot(client, oid, pgver);
	let policies = match pgver {
		PG_9_5..=PG_MAX => Policy::snapshot(client, oid, pgver),
		_ => BTreeMap::new(),
	};

	Some(
		Relation {
			ident: class.relname.clone(),
			attributes: atts,
			stats,
			indexes,
			constraints,
			rules,
			triggers,
			class,
			policies,
		}
	)
}
