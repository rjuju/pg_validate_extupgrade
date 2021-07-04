/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::{
	env,
	process,
};

use clap::{
	self,
	Arg,
	ErrorKind,
};

use postgres::{Client, NoTls};

mod extension;
use crate::extension::Extension;

#[macro_use]
mod compare;
use crate::compare::Compare;

mod pgdiff;
mod pgtype;

pub mod elog {
	pub const WARNING: u8 = 19;
	pub const ERROR: u8 = 21;

	fn lvl(level: u8) -> &'static str{
		match level {
			WARNING => "WARNING",
			ERROR => "ERROR",
			_ => { panic!("Unexpected level {}", level); }
		}
	}
	
	pub fn elog(level: u8, msg: &str) {
		eprintln!("{}: {}", lvl(level), msg);
	}
}

pub struct App {
	conninfo: Conninfo,
	extname: String,
	from: String,
	to: String
}

impl App {
	pub fn new() -> Self {
		let _host = match env::var("PGHOST") {
			Ok(h) => h,
			Err(_) => String::from("27.0.0.1"),
		};

		let _port = match env::var("PGPORT") {
			Ok(p) => p,
			Err(_) => String::from("5432"),
		};

		let _user = match env::var("PGUSER") {
			Ok(u) => u,
			Err(_) => whoami::username(),
		};

		let _db = match env::var("PGDATABASE") {
			Ok(d) => d,
			Err(_) => _user.clone(),
		};

		let matches = clap::App::new("pg_validate_extupgrade")
			.arg(Arg::with_name("extname")
				.short("e")
				.long("extname")
				.takes_value(true)
				.required(true)
				.help("extension to test")
			)
			.arg(Arg::with_name("from")
				.long("from")
				.takes_value(true)
				.required(true)
				.help("initial version of the extension")
			)
			.arg(Arg::with_name("to")
				.long("to")
				.takes_value(true)
				.required(true)
				.help("upgraded version of the extension")
			)
			.arg(Arg::with_name("host")
				.short("h")
				.long("host")
				.default_value(&_host)
				.help("database server host or socket directory")
			)
			.arg(Arg::with_name("port")
				.short("p")
				.long("port")
				.default_value(&_port)
				.help("database server port")
			)
			.arg(Arg::with_name("user")
				.short("U")
				.long("user")
				.default_value(&_user)
				.help("database user name")
			)
			.arg(Arg::with_name("dbname")
				.short("d")
				.long("dbname")
				.default_value(&_db)
				.help("database name")
			)
			.get_matches_from_safe(std::env::args_os())
			.unwrap_or_else(|e| e.exit());

		let conninfo = Conninfo::new_from(&matches).unwrap_or_else(|e| e.exit());

		let from = matches.value_of("from").unwrap();
		let to = matches.value_of("to").unwrap();

		if from == to {
			App::error(String::from("--from and --to must be different"));
		}

		App {
			conninfo,
			extname: String::from(matches.value_of("extname").unwrap()),
			from: String::from(from),
			to: String::from(to),
		}
	}

	fn connect(&self) -> Result<(Client, u32), postgres::Error> {
		let mut client = Client::connect(&self.conninfo.to_string(), NoTls)?;

		let rows = client.query("SHOW server_version_num", &[])?;
		let ver: &str = rows[0].get(0);

		println!("Connected, server version {}", ver);
		Ok((client, ver.parse().unwrap()))
	}

	fn error(msg: String) {
		println!("ERROR: {}", msg);
		process::exit(1);
	}

	fn check_ext(&self, client: &mut postgres::Client) {
		let rows = client.query("SELECT version \
			FROM pg_available_extension_versions \
			WHERE name = $1",
			&[&self.extname])
			.expect("Could not query pg_available_extension_versions");

		if rows.len() == 0 {
			App::error(format!("extension \"{}\" does not exits",
					self.extname));
		}

		let mut found_from = false;
		let mut found_to = false;
		let mut alt: Vec<&str> = Vec::new();

		for row in &rows {
			let ver: &str = row.get(0);

			if ver == self.from  {
				found_from = true;
			} else if ver == self.to {
				found_to = true;
			}
			else {
				alt.push(ver);
			}
		}

		if !found_from {
			App::error(format!("version \"{}\" of extension \"{}\" not found",
					self.from,
					self.extname));
		}

		if !found_to {
			App::error(format!("version \"{}\" of extension \"{}\" not found",
					self.to,
					self.extname));
		}
	}

	fn created(&self, client: &mut postgres::Transaction) {
			if let Err(e) = client.simple_query(
				&format!("CREATE EXTENSION {} VERSION '{}'",
					self.extname,
					self.to),
			) {
				App::error(e.to_string());
			};
	}

	fn updated(&self, client: &mut postgres::Transaction) {
			if let Err(e) = client.simple_query(
				&format!("DROP EXTENSION {e};\
					CREATE EXTENSION {e} VERSION '{}'; \
					ALTER EXTENSION {e} UPDATE TO '{}'",
					self.from,
					self.to,
					e = self.extname,)
			) {
				App::error(e.to_string());
			};
	}

	pub fn run(&self) -> Result<(), String> {
		let (mut client, pgver) = match self.connect() {
			Ok(c) => c,
			Err(e) => { return Err(e.to_string()); },
		};

		self.check_ext(&mut client);

		let mut transaction = client.transaction()
			.expect("Could not start a transaction");

		self.created(&mut transaction);
		let from = Extension::snapshot(&self.extname, &mut transaction, pgver);

		self.updated(&mut transaction);
		let to = Extension::snapshot(&self.extname, &mut transaction, pgver);

		let res = from.compare(&to);

		transaction.rollback().expect("Could not rollback the transaction");

		match res {
			None => Ok(()),
			Some(m) => Err(m.to_string()),
		}
	}
}

struct Conninfo {
	host: String,
	port: u16,
	user: String,
	dbname: String,
}

impl Conninfo {
	fn new_from(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
		let port = matches.value_of("port").unwrap();

		let port = match port.parse::<u16>() {
			Ok(p) => p,
			Err(_) => {
				return Err(clap::Error::with_description(
						&format!("Invalid port value \"{}\"", port),
						ErrorKind::InvalidValue));
			},
		};

		Ok(Conninfo {
			host: String::from(matches.value_of("host").unwrap()),
			port: port,
			user: String::from(matches.value_of("user").unwrap()),
			dbname: String::from(matches.value_of("dbname").unwrap()),
		})
	}
}

impl ToString for Conninfo {
	fn to_string(&self) -> String {
		format!("host={} port={} user={} dbname={}",
			self.host,
			self.port,
			self.user,
			self.dbname,
		)
	}
}

#[cfg(test)]
mod test {
	use std::collections::HashMap;
	use postgres::Row;
	use super::{*, compare::*, pgtype::*};

	DbStruct! {
		Attribute:attname:Attribute {
			attname: Name,
		}
	}

	DbStruct! {
		PgClass:relname:Relation {
			relname: Name,
			relkind: Char,
			relpersistence: Char,
			new_feature: Text = ("deparse(new_feature)") {PG_12..},
			deprecated_feature: bool {..PG_10},
			transient_feature: Char {PG_9_4..PG_10},
		}
	}
	CompareStruct! {
		Relation {
			attributes: Vec<Attribute>,
			class: PgClass,
		}
	}

	CompareStruct! {
		Extension {
			relations: Option<HashMap<String, Relation>>,
		}
	}

	fn get_extension(ident: &str, relations: Option<Vec<Relation>>)
		-> Extension
	{
		match relations {
			None => Extension {
				ident: String::from(ident),
				relations: None,
			},
			Some(v) => {
				let mut relations = HashMap::new();

				for r in v {
					relations.insert(r.class.relname.clone(), r);
				};

				Extension {
					ident: String::from(ident),
					relations: Some(relations),
				}
			}
		}
	}

	fn get_t1(pgver: u32) -> Relation {
		let new_feature = match pgver {
			PG_12..=PG_MAX => Some(String::from("some value")),
			_ => None,
		};
		let deprecated_feature = match pgver {
			PG_10..=PG_MAX => None,
			_ => Some(true),
		};
		let transient_feature = match pgver {
			PG_9_4..=90699 => Some('a' as i8),
			_ => None,
		};

		let class = PgClass {
			relname: String::from("t1"),
			relkind: 'r' as i8,
			relpersistence: 'p' as i8,
			new_feature,
			deprecated_feature,
			transient_feature,
		};

		Relation {
			ident: class.relname.clone(),
			attributes: vec![
				Attribute {
					attname: String::from("id"),
				}
			],
			class,
		}
	}

	#[test]
	fn test_tlist() {
		let t1_tlist = PgClass::tlist(PG_10);

		let mut exp_tlist = vec![
			String::from("relname"),
			String::from("relkind"),
			String::from("relpersistence"),
			String::from("NULL::text AS new_feature"),
			String::from("NULL::bool AS deprecated_feature"),
			String::from("NULL::\"char\" AS transient_feature"),
		];

		assert_eq!(exp_tlist, t1_tlist, "Target list for pg10 should not include \
			any of the optional feature");

		let t1_tlist = PgClass::tlist(PG_14);

		exp_tlist[3] =String::from("deparse(new_feature) AS new_feature");
		exp_tlist[4] =String::from("NULL::bool AS deprecated_feature");
		exp_tlist[5] =String::from("NULL::\"char\" AS transient_feature");

		assert_eq!(exp_tlist, t1_tlist, "Target list for pg14 should include \
			only \"new_feature\"");

		let t1_tlist = PgClass::tlist(PG_9_4);
		exp_tlist[3] =String::from("NULL::text AS new_feature");
		exp_tlist[4] =String::from("deprecated_feature");
		exp_tlist[5] =String::from("transient_feature");

		assert_eq!(exp_tlist, t1_tlist, "Target list for pg9.4 should include \
			\"deprecated_feature\" and \"transient_feature\"\n");

		let t1_tlist = PgClass::tlist(PG_9_3);
		exp_tlist[3] =String::from("NULL::text AS new_feature");
		exp_tlist[4] =String::from("deprecated_feature");
		exp_tlist[5] =String::from("NULL::\"char\" AS transient_feature");

		assert_eq!(exp_tlist, t1_tlist, "Target list for pg9.3 should include \
			only \"deprecated_feature\"");
	}

	#[test]
	fn compare_same_relation() {
		let mut msg = String::new();
		let t1 = get_t1(140000);

		t1.compare(&t1, &mut msg);

		assert_eq!("", msg, "Identical relation (v14) should not raise \
			anything\n{}", msg);

		let mut msg = String::new();
		let t1 = get_t1(430000);

		t1.compare(&t1, &mut msg);

		assert_eq!("", msg, "Identical relation (v43) should not raise \
			anything\n{}", msg);
	}

	#[test]
	fn compare_relation_ins_diff() {
		let mut msg = String::new();
		let mut t1_ins = get_t1(140000);
		let t1_upg = get_t1(140000);

		t1_ins.class.relkind = 'v' as i8;
		t1_ins.attributes[0].attname = String::from("ins_id");

		t1_ins.compare(&t1_upg, &mut msg);

		assert!(
			msg.contains("Relation t1 in relkind") &&
			msg.contains("- v") &&
			msg.contains("+ r") &&
			!msg.contains("PgClass")
			,
			"relkind change should be detected\n{}",
			msg
		);

		assert!(
			msg.contains("Attribute ins_id in attname") &&
			msg.contains("- ins_id") &&
			msg.contains("+ id") &&
			!msg.contains("PgClass")
			,
			"attribute attname change should be detected\n{}",
			msg
		);
	}

	#[test]
	fn compare_relation_upg_diff() {
		let mut msg = String::new();
		let t1_ins = get_t1(430000);
		let mut t1_upg = get_t1(430000);

		t1_upg.class.relkind = 'v' as i8;
		t1_upg.attributes[0].attname = String::from("upg_id");

		t1_ins.compare(&t1_upg, &mut msg);

		assert!(
			msg.contains("Relation t1 in relkind") &&
			msg.contains("- r") &&
			msg.contains("+ v") &&
			!msg.contains("PgClass")
			,
			"relkind change should be detected\n{}",
			msg
		);

		assert!(
			msg.contains("Attribute id in attname") &&
			msg.contains("- id") &&
			msg.contains("+ upg_id") &&
			!msg.contains("PgClass")
			,
			"attribute attname change should be detected\n{}",
			msg
		);
	}

	#[test]
	fn compare_relation_pgver_diff() {
		let mut msg = String::new();
		let t1_ins = get_t1(PG_10);
		let t1_upg = get_t1(PG_14);

		t1_ins.compare(&t1_upg, &mut msg);

		assert_ne!("", msg, "Should find mismatch comparing ins to upg");
		assert_eq!(true,
			msg.contains("Relation t1 in new_feature") &&
			msg.contains("installed has no value") &&
			msg.contains("upgraded has")
			,
			"Mismatch in optional field (missing in ins) should be \
				detected\n{}",
			msg
		);

		let mut msg = String::new();
		let t1_ins = get_t1(PG_14);
		let t1_upg = get_t1(PG_10);

		t1_ins.compare(&t1_upg, &mut msg);

		assert_ne!("", msg, "Should find mismatch comparing upg to ins");
		assert_eq!(true,
			msg.contains("Relation t1 in new_feature") &&
			msg.contains("upgraded has no value") &&
			msg.contains("installed has")
			,
			"Mismatch in optional field (missing in upg) should be \
				detected\n{}",
			msg
		);
	}

	#[test]
	fn compare_relation_opt_diff() {
		let mut msg = String::new();
		let mut t1_ins = get_t1(430000);
		let t1_upg = get_t1(430000);

		t1_ins.class.new_feature = Some(String::from("ins some value"));
		t1_ins.compare(&t1_upg, &mut msg);

		assert!(
			msg.contains("Relation t1 in new_feature") &&
			msg.contains("- ins some value") &&
			msg.contains("+ some value")
			,
			"Mismatch in optional field (changed in ins) should be \
				detected\n{}",
			msg
		);

		let mut msg = String::new();
		let t1_ins = get_t1(430000);
		let mut t1_upg = get_t1(430000);

		t1_upg.class.new_feature = Some(String::from("upg some value"));
		t1_ins.compare(&t1_upg, &mut msg);

		assert!(
			msg.contains("Relation t1 in new_feature") &&
			msg.contains("- some value") &&
			msg.contains("+ upg some value")
			,
			"Mismatch in optional field (changed in upg) should be \
				detected\n{}",
			msg
		);
	}

	#[test]
	fn compare_same_ext() {
		let mut msg = String::new();
		let ext_ins = get_extension("empty_ext", None);

		ext_ins.compare(&ext_ins, &mut msg);

		assert_eq!("", msg,
			"Two empty extensions should be identical\n{}",
			msg);

		let mut msg = String::new();
		let ext_ins = get_extension("empty_ext", Some(vec![]));

		ext_ins.compare(&ext_ins, &mut msg);

		assert_eq!("", msg,
			"Two extensions with empty rel list should be identical\n{}",
			msg);

		let mut msg = String::new();
		let t1 = get_t1(140000);
		let ext_ins = get_extension("ext_1_rel", Some(vec![t1]));

		ext_ins.compare(&ext_ins, &mut msg);

		assert_eq!("", msg,
			"Two extensions with same 1 rel should be identical\n{}",
			msg);
	}

	#[test]
	fn compare_ext_diff_nb_rels() {
		let mut msg = String::new();
		let t1_a = get_t1(140000);
		let t1_b = get_t1(140000);
		let mut t2 = get_t1(140000);

		t2.class.relname = String::from("t2");

		let ext_ins = get_extension("ext_1_rel", Some(vec![t1_a]));
		let ext_upg = get_extension("ext_2_rel", Some(vec![t1_b, t2]));

		ext_ins.compare(&ext_upg, &mut msg);

		assert!(msg.contains("Upgraded version has 1 more Relation") &&
			msg.contains("- t2"),
			"Should detect that upgraded extension has 1 more rel\n{}",
			msg);

		let mut msg = String::new();

		ext_upg.compare(&ext_ins, &mut msg);

		assert!(msg.contains("Installed version has 1 more Relation") &&
			msg.contains("- t2"),
			"Should detect that installed extension has 1 more rel\n{}",
			msg);
	}
}
