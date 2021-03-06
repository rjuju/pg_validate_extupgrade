-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "CREATE EXTENSION pg_broken_extupgrade" to load this file. \quit

CREATE TABLE tbl0(id integer, val text);
CREATE INDEX ON tbl0 (id) WHERE (id > 0);
CREATE TABLE broken_tbl1(id int, val text);
CREATE TABLE tbl2(broken_id int, val text COLLATE "C");
--CREATE TABLE tbl3(id int, val varchar(30));
CREATE TABLE tbl3(id serial, val varchar(30));
CREATE TABLE tbl4(id int, val varchar(30));
CREATE TABLE missing1(id integer);
CREATE UNLOGGED TABLE logged(id integer, val text);
ALTER TABLE logged SET (fillfactor = 80);
REVOKE SELECT ON logged FROM public;
