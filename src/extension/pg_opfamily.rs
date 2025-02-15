/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2025 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use postgres::{Row, Transaction};
use std::collections::BTreeMap;

use crate::{compare::*, elog::*, pgdiff::SchemaDiff, pgtype::*, DbStruct};

DbStruct! {
    OpFamily:opfname:OpFamily {
        opfname: Text = ("n.nspname || '.' || opf.opfname || ' USING ' || am.amname"),
        opfmethod: Name = ("am.amname"),
        opfowner: Name = ("r.rolname"),
    }
}

impl OpFamily {
    pub fn snapshot<'a>(
        client: &mut Transaction,
        oids: Vec<u32>,
        pgver: u32,
    ) -> BTreeMap<String, OpFamily> {
        let mut opfs = BTreeMap::new();

        for oid in oids {
            let opf = snap_one_opf(client, oid, pgver);
            opfs.insert(opf.opfname.clone(), opf);
        }

        opfs
    }
}

pub fn snap_one_opf(client: &mut Transaction, oid: u32, pgver: u32) -> OpFamily {
    let sql = format!(
        "SELECT {} \
        FROM pg_opfamily opf \
        JOIN pg_namespace n ON n.oid = opf.opfnamespace \
        JOIN pg_am am ON am.oid = opf.opfmethod \
        JOIN pg_roles r ON r.oid = opf.opfowner \
        WHERE opf.oid = $1",
        OpFamily::tlist(pgver).join(", "),
    );

    let row = match client.query_one(&sql[..], &[&oid]) {
        Ok(r) => r,
        Err(e) => {
            elog(ERROR, &format!("{}", e));
            panic!();
        }
    };

    OpFamily::from_row(&row)
}
