/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2025 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use postgres::{Row, Transaction};
use std::collections::BTreeMap;

use crate::{compare::*, elog::*, pgdiff::SchemaDiff, pgtype::*, proc_prototype, DbStruct};

DbStruct! {
    EventTrigger:evtname:EventTrigger {
        evtname: Name,
        evtevent: Name,
        evtowner: Name = ("r.rolname"),
        evtfoid: Text = (proc_prototype!("evtfoid")),
        evtenabled: Char,
        evttags: Option<List>,
        comment: Option<Text> = ("obj_description(t.oid, 'pg_event_trigger')"),
    }
}

impl EventTrigger {
    pub fn snapshot<'a>(
        client: &mut Transaction,
        oids: Vec<u32>,
        pgver: u32,
    ) -> BTreeMap<String, EventTrigger> {
        let mut evt_trgs = BTreeMap::new();

        for oid in oids {
            let trg = snap_one_evt_trg(client, oid, pgver);
            evt_trgs.insert(trg.evtname.clone(), trg);
        }

        evt_trgs
    }
}

pub fn snap_one_evt_trg(client: &mut Transaction, relid: u32, pgver: u32) -> EventTrigger {
    let sql = format!(
        "SELECT {} \
        FROM pg_event_trigger t \
        JOIN pg_roles r on r.oid = t.evtowner \
        WHERE t.oid = $1",
        EventTrigger::tlist(pgver).join(", "),
    );

    let row = match client.query_one(&sql[..], &[&relid]) {
        Ok(r) => r,
        Err(e) => {
            elog(ERROR, &format!("{}", e));
            panic!();
        }
    };

    EventTrigger::from_row(&row)
}
