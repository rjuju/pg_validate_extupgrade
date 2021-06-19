/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::{
	env,
	ffi::OsString,
};

use clap::{
	self,
	Arg,
	ErrorKind,
};

use postgres::{Client, NoTls};

pub struct App {
	conninfo: Conninfo,
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

		App {
			conninfo,
		}
	}

	fn connect(&self) -> Result<Client, postgres::Error> {
		let mut client = Client::connect(&self.conninfo.to_string(), NoTls)?;

		let rows = client.query("SHOW server_version", &[])?;
		let ver: &str = rows[0].get(0);

		println!("Connected, server version {}", ver);
		Ok(client)
	}

	pub fn run(&self) -> Result<(), String> {
		let client = match self.connect() {
			Ok(c) => c,
			Err(e) => { return Err(e.to_string()); },
		};

		println!("FIXME");

		Ok(())
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
