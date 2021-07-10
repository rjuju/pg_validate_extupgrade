-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "CREATE EXTENSION pg_broken_extupgrade" to load this file. \quit

CREATE TABLE tbl0(id integer, val text);
CREATE STATISTICS tbl0_stats ON id, val, (id *2) FROM tbl0;
CREATE STATISTICS tbl0_stats_n (dependencies) ON (id + 1), val FROM tbl0;
CREATE INDEX ON tbl0 (id) WHERE (id > 0);
CREATE TABLE broken_tbl1(id int, val text);
CREATE TABLE tbl2(broken_id int, val text COLLATE "C");
--CREATE TABLE tbl3(id int, val varchar(30));
CREATE TABLE tbl3(id serial, val varchar(30)) PARTITION BY LIST(id);
CREATE TABLE tbl4(id int, val varchar(30));
CREATE TABLE missing1(id integer);
CREATE UNLOGGED TABLE logged(id integer, val text);
ALTER TABLE logged SET (fillfactor = 80);
REVOKE SELECT ON logged FROM public;
CREATE TABLE main(id integer PRIMARY KEY);
CREATE TABLE ref(id integer REFERENCES main (id));
CREATE TABLE main2(id integer PRIMARY KEY, val text CHECK (length(val) > 2));
CREATE TABLE ref2(id integer references main2 (id) ON UPDATE cascade ON DELETE cascade);
