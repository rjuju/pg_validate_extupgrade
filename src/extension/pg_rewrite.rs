use postgres::{Row, Transaction};
use std::collections::BTreeMap;

use crate::{compare::*, pgdiff::SchemaDiff, pgtype::*, DbStruct};

DbStruct! {
    Rewrite:rulename:Rule {
        rulename: Name,
        ev_enabled: Char,
        ruledef: Text = ("pg_get_ruledef(oid)"),
        comment: Option<Text> = ("obj_description(oid, 'pg_rewrite')"),
    }
}

impl Rewrite {
    pub fn snapshot(client: &mut Transaction, relid: u32, pgver: u32) -> BTreeMap<String, Rewrite> {
        let mut rewrites = BTreeMap::new();

        let sql = format!(
            "SELECT {} \
            FROM pg_rewrite \
            WHERE ev_class = $1",
            Rewrite::tlist(pgver).join(", "),
        );

        let rows = client
            .query(&sql[..], &[&relid])
            .expect("Could net get pg_rewrite rows");

        for row in &rows {
            let rewrite = Rewrite::from_row(row);
            rewrites.insert(rewrite.rulename.clone(), rewrite);
        }

        rewrites
    }
}
