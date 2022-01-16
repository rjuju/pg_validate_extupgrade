/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2022 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::{
	collections::BTreeMap,
	env,
	ffi::OsStr,
	fs,
	path::Path,
	process,
};

use clap::{
	self,
	Arg,
	ErrorKind,
};

use postgres::{Client, NoTls};
use serde::Deserialize;
use toml::Value;

mod extension;
use crate::extension::Extension;
mod guc;

#[macro_use]
mod compare;
use crate::compare::{Compare, PG_9_6};

mod pgdiff;
mod pgtype;
use pgtype::{ExecutedQueries, Guc};

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

use elog::*;

#[derive(Deserialize)]
struct Config {
	extname: Option<String>,
	from: Option<String>,
	to: Option<String>,
	host: Option<String>,
	port: Option<u16>,
	user: Option<String>,
	dbname: Option<String>,
	extra_queries: Option<Vec<String>>,
}

impl<'a> Config {
	fn new() -> Self {
		Config {
			extname: None,
			from: None,
			to: None,
			host: None,
			port: None,
			user: None,
			dbname: None,
			extra_queries: None,
		}
	}

	fn apply_matches(&mut self, matches: &clap::ArgMatches) {
		if self.extname.is_none() || matches.occurrences_of("extname") != 0 {
			self.extname = match matches.value_of("extname") {
				Some(e) => Some(String::from(e)),
				None => {
					clap::Error::with_description(
							"--extname is required",
							ErrorKind::MissingRequiredArgument).exit();
				},
			};
		}

		if self.from.is_none() || matches.occurrences_of("from") != 0 {
			self.from = match matches.value_of("from") {
				Some(e) => Some(String::from(e)),
				None => {
					clap::Error::with_description(
							"--from is required",
							ErrorKind::MissingRequiredArgument).exit();
				},
			};
		}

		if self.to.is_none() || matches.occurrences_of("to") != 0 {
			self.to = match matches.value_of("to") {
				Some(e) => Some(String::from(e)),
				None => {
					clap::Error::with_description(
							"--to is required",
							ErrorKind::MissingRequiredArgument).exit();
				},
			};
		}

		if self.host.is_none() || matches.occurrences_of("host") != 0 {
			self.host = Some(
				String::from(matches.value_of("host").unwrap())
			);
		}

		if self.port.is_none() || matches.occurrences_of("port") != 0 {
			let port = matches.value_of("port").unwrap();
			let port = match port.parse::<u16>() {
				Ok(p) => p,
				Err(_) => {
					clap::Error::with_description(
						&format!("Invalid port value \"{}\"", port),
						ErrorKind::InvalidValue).exit();
				},
			};

			self.port = Some(port);
		}

		if self.user.is_none() || matches.occurrences_of("user") != 0 {
			self.user = Some(
				String::from(matches.value_of("user").unwrap())
			);
		}

		if self.dbname.is_none() || matches.occurrences_of("dbname") != 0 {
			self.dbname = Some(
				String::from(matches.value_of("dbname").unwrap())
			);
		}
	}

	fn check_config_keys<I>(keys: I, format: &str)
		where I: Iterator<Item = &'a String>
	{
		for k in keys {
			match &k[..] {
				"extname" | "from" | "to" | "host" | "port" | "user" |
					"dbname" | "extra_queries" => {
				},
				_ => {
					elog(WARNING,
						&format!("Unexpected {} key \"{}\"", format, k));
				}
			}
		}
	}

	fn from_toml(lines: &str, filename: &str) -> Result<Self, String> {
		let config: Config = match toml::from_str(lines) {
			Ok(c) => { c },
			Err(e) => { return Err(format!("Could not parse \"{}\":\n{}",
					filename, e)); },
		};

		// Do an explicit parse of the input to warn about unexpected keys
		let toml = lines.parse::<Value>().unwrap();
		match toml {
			Value::Table(m) => {
				Config::check_config_keys(m.keys(), "TOML");
			},
			_ => {
				return Err(format!("Unexpected TOML Value:\n {:#?}", toml));
			},
		};

		Ok(config)
	}

	fn from_json(lines: &str, filename: &str) -> Result<Self, String> {
		let config: Config = match serde_json::from_str(lines) {
			Ok(c) => { c },
			Err(e) => { return Err(format!("Could not parse \"{}\":\n{}",
					filename, e)); },
		};

		// Do an explicit parse of the input to warn about unexpected keys
		let json: serde_json::Value = serde_json::from_str(lines).unwrap();
		match json {
			serde_json::Value::Object(m) => {
				Config::check_config_keys(m.keys(), "JSON");
			},
			_ => {
				return Err(format!("Unexpected JSON Value:\n {:#?}", json));
			}
		}

		Ok(config)
	}

	fn from_file(filename: &str) -> Result<Self, String> {
		let content = match fs::read_to_string(filename) {
			Ok(c) => { c },
			Err(e) => {
				return Err(format!("Could not read \"{}\": {}",
						filename, e));
			}
		};

		let file_ext = Path::new(filename).extension().and_then(OsStr::to_str);
		let config = match file_ext {
			Some(e) => {
				match e {
					"toml" => {
						Config::from_toml(&content, filename)
					},
					"json" => {
						Config::from_json(&content, filename)
					},
					_ => {
						return Err(format!("Unsupported extension: {}", e));
					},
				}
			},
			None => {
				return Err(format!("No extension found for file \"{}\"",
						filename));
			},
		};

		config
	}
}

pub struct App {
	extname: String,
	from: String,
	to: String,
	host: String,
	port: u16,
	user: String,
	dbname: String,
	extra_queries: Vec<String>,
}

impl App {
	pub fn new() -> Self {
		let _host = match env::var("PGHOST") {
			Ok(h) => h,
			Err(_) => String::from("localhost"),
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
			.author("Julien Rouhaud <rjuju123 (at) gmail (dot) com")
			.about("Tool to validate PostgreSQL extension upgrade scripts.")
			.max_term_width(100)
			.arg(Arg::with_name("extname")
				.short("e")
				.long("extname")
				.takes_value(true)
				.help("extension to test")
			)
			.arg(Arg::with_name("from")
				.long("from")
				.takes_value(true)
				.help("initial version of the extension")
			)
			.arg(Arg::with_name("to")
				.long("to")
				.takes_value(true)
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
			.arg(Arg::with_name("filename")
				.short("c")
				.long("config")
				.takes_value(true)
				.help("configuration file name.  Supported extension: \
				.toml and .json")
			)
			.get_matches_from_safe(std::env::args_os())
			.unwrap_or_else(|e| e.exit());

		let mut config = match matches.value_of("filename") {
			Some(f) => Config::from_file(f).unwrap_or_else(|e| {
				clap::Error::with_description(&e,
					ErrorKind::Io).exit();
			}),
			None => Config::new(),
		};

		if config.extra_queries.is_none() {
			config.extra_queries = Some(vec![]);
		}

		config.apply_matches(&matches);

		if config.from == config.to {
			clap::Error::with_description(
				"--from and --to must be different",
				ErrorKind::InvalidValue).exit();
		}

		App {
			extname: config.extname.unwrap(),
			from: config.from.unwrap(),
			to: config.to.unwrap(),
			host: config.host.unwrap(),
			port: config.port.unwrap(),
			user: config.user.unwrap(),
			dbname: config.dbname.unwrap(),
			extra_queries: config.extra_queries.unwrap(),
		}
	}

	fn conninfo(&self) -> String {
		format!("host={} port={} user={} dbname={}",
			self.host,
			self.port,
			self.user,
			self.dbname)
	}

	fn connect(&self) -> Result<(Client, u32), postgres::Error> {
		let conninfo = self.conninfo();
		let mut client = Client::connect(&conninfo, NoTls)?;

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

			if ver == self.from {
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

	fn install_version<'a>(&self, client: &mut postgres::Transaction, pgver: u32,
		extver: &'a str) -> (Guc, Guc)
	{
		let cascade = match pgver >= PG_9_6 {
			true => "CASCADE",
			false => "",
		};

		let guc_pre = Guc::snapshot(client, String::from(extver));

		if let Err(e) = client.simple_query(
			&format!("CREATE EXTENSION {} VERSION '{}' {} ;",
				self.extname,
				extver,
				cascade),
		) {
			App::error(e.to_string());
		};

		let guc_post = Guc::snapshot(client, String::from(extver));

		client.execute("RESET ALL", &[])
			.expect("Could not execute RESET ALL");

		(guc_pre, guc_post)
	}

	fn update_version<'a>(&self, client: &mut postgres::Transaction)
		-> (Guc, Guc)
	{
		let guc_ver = String::from(format!("{}--{}",
					self.from,
					self.to,
			));

		let guc_pre = Guc::snapshot(client, guc_ver.clone());

		if let Err(e) = client.simple_query(
			&format!("ALTER EXTENSION {} UPDATE TO '{}'",
					self.extname,
					self.to)
		) {
			App::error(e.to_string());
		};

		let guc_post = Guc::snapshot(client, guc_ver);

		client.execute("RESET ALL", &[])
			.expect("Could not execute RESET ALL");

		(guc_pre, guc_post)
	}

	fn run_extra_queries(&self, client: &mut postgres::Transaction)
	-> ExecutedQueries
	{
		let mut result = BTreeMap::new();

		for query in &self.extra_queries {
			let mut out = String::new();
			let len;

			let mut savepoint = client.transaction()
				.expect("Coult not create a savepoint");
			let rows = &savepoint.query(&query[..], &[]);
			savepoint.rollback().expect("Could not rollback savepoint");

			match rows {
				Ok(rows) => {
					len = rows.len();
					for row in rows {
						out.push_str(&row_to_string(row, &query));
					}
				},
				Err(e) => {
					len = 0;
					out.push_str(&format!("Could not execute query:\n{}\n", e));
				}
			}

			result.insert(query.clone(), (len, out));
		}

		ExecutedQueries::new_from(result)
	}

	pub fn run(&self) -> Result<(), String> {
		let (mut client, pgver) = match self.connect() {
			Ok(c) => c,
			Err(e) => { return Err(e.to_string()); },
		};

		self.check_ext(&mut client);

		let mut transaction = client.transaction()
			.expect("Could not start a transaction");

		let mut result = String::new();

		// First round installing directly the target version
		let (pre, post) = self.install_version(&mut transaction, pgver,
			&self.to);

		if let Some(d) = pre.compare(&post) {
			result.push_str(&d.to_string());
		}

		let mut from = Extension::snapshot(&self.extname, &mut transaction,
			pgver);
		from.set_extra_queries(self.run_extra_queries(&mut transaction));

		// Remove the extension
		transaction.simple_query(&format!("DROP EXTENSION {}", self.extname))
			.expect("Could not execute DROP EXTENSION");

		// Second round, install source version and update it
		let (pre, post) = self.install_version(&mut transaction, pgver,
			&self.from);
		if let Some(d) = pre.compare(&post) {
			result.push_str(&d.to_string());
		}

		 let (pre, post) = self.update_version(&mut transaction);
		 if let Some(d) = pre.compare(&post) {
			result.push_str(&d.to_string());
		}

		let mut to = Extension::snapshot(&self.extname, &mut transaction, pgver);
		to.set_extra_queries(self.run_extra_queries(&mut transaction));

		let res = from.compare(&to);

		transaction.rollback().expect("Could not rollback the transaction");

		if let Some(m) = res {
			result.push_str(&m.to_string());
		};

		match result.len() {
			0 => Ok(()),
			_ => Err(result),
		}
	}
}

fn row_to_string(row: &postgres::row::Row, query: &str) -> String {
	let mut line = String::new();

	macro_rules! DeparseField {
		($i:ident, $col:ident, $($pg:ident => $rust:ty ),*,) => {
			let val = match $col.type_() {
			$(
				&postgres::types::Type::$pg => {
					match row.try_get::<_, $rust>($i) {
						Ok(v) => v.to_string(),
						Err(_) => String::from("<NULL>"),
					}
				},
			)*
				_ => {
					App::error(
						format!("Type \"{}\" for column \"{}\" not handled. \
						Query:\n{}",
							$col.type_(), $col.name(), query));
					format!("unhandled")
				},
			};
			line.push_str(&format!("{}: {}\n", $col.name(), val));
		}
	}

	for (i, col) in row.columns().iter().enumerate() {
		DeparseField!(i,col,
			BOOL    => bool,
			CHAR    => i8,
			INT2    => i16,
			INT4    => i32,
			INT8    => i64,
			FLOAT4  => f32,
			FLOAT8  => f64,
			NAME    => String,
			OID     => u32,
			TEXT    => String,
			VARCHAR => String,
		);
	}
	line.push_str("\n");

	line
}

#[cfg(test)]
mod test {
	use std::collections::BTreeMap;
	use postgres::Row;
	use super::{*, compare::*, pgdiff::*, pgtype::*};

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
			relations: Option<BTreeMap<String, Relation>>,
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
				let mut relations = BTreeMap::new();

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
		let t1 = get_t1(PG_14);

		let msg = t1.compare(&t1);

		assert!(msg.is_none(), "Identical relation (v14) should not raise \
			anything\n{}", msg.unwrap().to_string());

		let t1 = get_t1(430000);

		let msg = t1.compare(&t1);

		assert!(msg.is_none(), "Identical relation (v43) should not raise \
			anything\n{}", msg.unwrap().to_string());
	}

	#[test]
	fn compare_relation_ins_diff() {
		let mut t1_ins = get_t1(PG_14);
		let t1_upg = get_t1(PG_14);

		t1_ins.class.relkind = 'v' as i8;
		t1_ins.attributes[0].attname = String::from("ins_id");

		let msg = t1_ins.compare(&t1_upg).expect("Should find differences")
			.to_string();

		assert!(
			msg.contains("for Relation t1") &&
			msg.contains("in relkind") &&
			msg.contains("- v") &&
			msg.contains("+ r") &&
			!msg.contains("PgClass")
			,
			"relkind change should be detected\n{}",
			msg
		);

		assert!(
			msg.contains("Attribute ins_id") &&
			msg.contains("in attname") &&
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
		let t1_ins = get_t1(430000);
		let mut t1_upg = get_t1(430000);

		t1_upg.class.relkind = 'v' as i8;
		t1_upg.attributes[0].attname = String::from("upg_id");

		let msg = t1_ins.compare(&t1_upg).expect("Should find differences")
			.to_string();

		assert!(
			msg.contains("for Relation t1") &&
			msg.contains("in relkind") &&
			msg.contains("- r") &&
			msg.contains("+ v") &&
			!msg.contains("PgClass")
			,
			"relkind change should be detected\n{}",
			msg
		);

		assert!(
			msg.contains("for Attribute id") &&
			msg.contains("in attname") &&
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
		let t1_ins = get_t1(PG_10);
		let t1_upg = get_t1(PG_14);

		let msg = t1_ins.compare(&t1_upg).expect("Should find differences")
			.to_string();

		assert_eq!(true,
			msg.contains("for Relation t1") &&
			msg.contains("in new_feature") &&
			msg.contains("installed has no value") &&
			msg.contains("upgraded has")
			,
			"Mismatch in optional field (missing in ins) should be \
				detected\n{}",
			msg
		);

		let t1_ins = get_t1(PG_14);
		let t1_upg = get_t1(PG_10);

		let msg = t1_ins.compare(&t1_upg).expect("Should find differences")
			.to_string();

		assert_eq!(true,
			msg.contains("for Relation t1") &&
			msg.contains("in new_feature") &&
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
		let mut t1_ins = get_t1(430000);
		let t1_upg = get_t1(430000);

		t1_ins.class.new_feature = Some(String::from("ins some value"));
		let msg = t1_ins.compare(&t1_upg).expect("Should find differences")
			.to_string();

		assert!(
			msg.contains("for Relation t1") &&
			msg.contains("- in new_feature") &&
			msg.contains("- ins some value") &&
			msg.contains("+ some value")
			,
			"Mismatch in optional field (changed in ins) should be \
				detected\n{}",
			msg
		);

		let t1_ins = get_t1(430000);
		let mut t1_upg = get_t1(430000);

		t1_upg.class.new_feature = Some(String::from("upg some value"));
		let msg = t1_ins.compare(&t1_upg).expect("Should find differences")
			.to_string();

		assert!(
			msg.contains("for Relation t1") &&
			msg.contains("in new_feature") &&
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
		let ext_ins = get_extension("empty_ext", None);

		let msg = ext_ins.compare(&ext_ins);

		match msg {
			Some(m) => {
			assert!(false,
				"Two empty extensions should be identical\n{}",
				m.to_string());
			},
			None => {},
		}

		let ext_ins = get_extension("empty_ext", Some(vec![]));

		let msg = ext_ins.compare(&ext_ins);

		match msg {
			Some(m) => {
			assert!(false,
				"Two extensions with empty rel list should be identical\n{}",
				m.to_string());
			},
			None => {},
		}

		let t1 = get_t1(PG_14);
		let ext_ins = get_extension("ext_1_rel", Some(vec![t1]));

		let msg = ext_ins.compare(&ext_ins);

		match msg {
			Some(m) => {
			assert!(false,
				"Two extensions with same 1 rel should be identical\n{}",
				m.to_string());
			},
			None => {},
		}
	}

	#[test]
	fn compare_ext_diff_nb_rels() {
		let t1_a = get_t1(PG_14);
		let t1_b = get_t1(PG_14);
		let mut t2 = get_t1(PG_14);

		t2.class.relname = String::from("t2");

		let ext_ins = get_extension("ext_1_rel", Some(vec![t1_a]));
		let ext_upg = get_extension("ext_2_rel", Some(vec![t1_b, t2]));

		let msg = ext_ins.compare(&ext_upg).expect("Should find differences")
			.to_string();

		assert!(msg.contains("upgraded has 1 more Relation (2) \
			than installed (1)") &&
			msg.contains("- t2"),
			"Should detect that upgraded extension has 1 more rel\n{}",
			msg);

		let msg = ext_upg.compare(&ext_ins).expect("Should find differences")
			.to_string();

		assert!(msg.contains("installed has 1 more Relation (2) \
			than upgraded (1)") &&
			msg.contains("- t2"),
			"Should detect that installed extension has 1 more rel\n{}",
			msg);
	}

	#[test]
	fn compare_unified_diff() {
		let mut t1_a = get_t1(PG_14);
		let mut t1_b = get_t1(PG_14);

		t1_a.class.new_feature = Some(String::from("some data\n-- something\nother"));
		t1_b.class.new_feature = Some(String::from("some data\n-- some thing\nother"));

		let msg = t1_a.compare(&t1_b).expect("Should find differences")
			.to_string();

		assert!(msg.contains("- mismatch found for Relation t1:") &&
			msg.contains("in new_feature:") &&
			msg.contains("--- installed\n+++ upgraded\n") &&
			msg.contains("some data\n--- something\n+-- some thing\n other"),
			"Should find a unified diff:\n {:#?}", msg);
	}
}
