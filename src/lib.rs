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

	fn connect(&self) -> Result<Client, postgres::Error> {
		let mut client = Client::connect(&self.conninfo.to_string(), NoTls)?;

		let rows = client.query("SHOW server_version", &[])?;
		let ver: &str = rows[0].get(0);

		println!("Connected, server version {}", ver);
		Ok(client)
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
		let mut client = match self.connect() {
			Ok(c) => c,
			Err(e) => { return Err(e.to_string()); },
		};

		self.check_ext(&mut client);

		let mut transaction = client.transaction()
			.expect("Could not start a transaction");

		self.created(&mut transaction);
		let from = Extension::snapshot(&self.extname, &mut transaction);

		self.updated(&mut transaction);
		let to = Extension::snapshot(&self.extname, &mut transaction);

		let mut res = String::new();
		from.compare(&to, &mut res);

		transaction.rollback().expect("Could not rollback the transaction");

		if res == "" {
			Ok(())
		} else {
			Err(res)
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
