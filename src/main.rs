/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::process;
use pg_validate_extupgrade::App;

fn main() {
    App::new().run().unwrap_or_else(|e| {
		println!("ERROR:\n{}", e);
		process::exit(1);
	});

	println!("No difference found.");
}
