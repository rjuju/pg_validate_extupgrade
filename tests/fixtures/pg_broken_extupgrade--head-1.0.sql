-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "CREATE EXTENSION pg_broken_extupgrade" to load this file. \quit

CREATE TABLE tbl0(id integer, val text);
CREATE STATISTICS tbl0_stats ON id, val FROM tbl0;
CREATE STATISTICS tbl0_stats_n (ndistinct) ON (id + 1), val FROM tbl0;
CREATE INDEX ON tbl0 (id);
CREATE TABLE tbl1(id integer, val text);
CREATE TABLE tbl2(id integer, val text);
CREATE TABLE tbl3(id integer, val text);
CREATE TABLE tbl4(id integer, val varchar(20));
CREATE TABLE logged(id integer, val text);
CREATE TABLE missing2(id integer);
CREATE TABLE papart();
CREATE TABLE main(id integer);
CREATE TABLE ref(id integer);
CREATE TABLE main2(id integer primary key, val text CHECK (length(val) > 1));
CREATE TABLE ref2(id integer references main2 (id) ON UPDATE no action ON DELETE restrict);
CREATE TABLE options_1(id integer);
ALTER TABLE options_1 SET (fillfactor = 80, toast_tuple_target = 8100);
CREATE TABLE options_2(id integer);
ALTER TABLE options_2 SET (fillfactor = 80, toast_tuple_target = 8100);
CREATE VIEW v1 AS select 1;
CREATE TABLE tbl_rewrite(id integer);
CREATE RULE r1 AS ON UPDATE TO tbl_rewrite DO ALSO DELETE FROM v1;
CREATE RULE r2 AS ON INSERT TO tbl_rewrite DO NOTHING;
CREATE RULE r3 AS ON DELETE TO tbl_rewrite DO NOTHING;
