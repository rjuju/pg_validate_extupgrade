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
