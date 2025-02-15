use postgres::{Row, Transaction};
use std::collections::BTreeMap;

use crate::{compare::*, pgdiff::SchemaDiff, pgtype::*, DbStruct};

pub const PG_MIN_VER: u32 = PG_10;

DbStruct! {
    ExtendedStatistic:stxname:ExtendedStatistic {
        stxname: Text = ("s.stxnamespace::regnamespace || '.' || s.stxname"),
        stxowner: Name = ("r.rolname"),
        columns: Text = ("pg_get_statisticsobjdef_columns(s.oid)") {PG_14..},
        stxkeys: List = ("(SELECT array_agg(attname) \
            FROM unnest(s.stxkeys) u(attnum) \
            JOIN pg_attribute a ON (s.stxrelid = a.attrelid AND \
                a.attnum = u.attnum AND NOT a.attisdropped))") {..PG_14},
        stxkind: Vec<Char>,
        stxstattarget: Integer = ("s.stxstattarget::integer") {PG_13..},
        comment: Option<Text> = ("obj_description(s.oid, 'pg_statistic_ext')"),
    }
}

impl ExtendedStatistic {
    pub fn snapshot(
        client: &mut Transaction,
        relid: u32,
        pgver: u32,
    ) -> BTreeMap<String, ExtendedStatistic> {
        let mut statistics = BTreeMap::new();

        assert!(pgver >= PG_MIN_VER);

        let sql = format!(
            "SELECT {} \
            FROM pg_statistic_ext s \
            JOIN pg_roles r on r.oid = s.stxowner \
            WHERE stxrelid = $1",
            ExtendedStatistic::tlist(pgver).join(", "),
        );

        let rows = client
            .query(&sql[..], &[&relid])
            .expect("Could net get pg_statistic_ext rows");

        for row in &rows {
            let stat = ExtendedStatistic::from_row(row);

            statistics.insert(stat.stxname.clone(), stat);
        }

        statistics
    }
}
