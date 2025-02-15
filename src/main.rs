/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2025 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use pg_validate_extupgrade::{elog::*, App};
use std::process;

fn main() {
    App::new().run().unwrap_or_else(|e| {
        elog(ERROR, &format!("Differences found:\n{}", e));
        process::exit(1);
    });

    println!("No difference found.");
}
