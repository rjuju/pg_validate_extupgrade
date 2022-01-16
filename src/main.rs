/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2022 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::process;
use pg_validate_extupgrade::{App, elog::*};

fn main() {
    App::new().run().unwrap_or_else(|e| {
		elog(ERROR, &format!("Differences found:\n{}", e));
		process::exit(1);
	});

	println!("No difference found.");
}
