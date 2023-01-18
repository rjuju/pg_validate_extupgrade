-- This program is open source, licensed under the PostgreSQL License.
-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "ALTER EXTENSION pg_broken_extupgrade UPDATE" to load this file. \quit
SET work_mem = '321kB';
SET LOCAL maintenance_work_mem TO '6666';
SET LOCAL client_min_messages = WARNING;

-- those 3 are just to validate that the am is used in the identifier
ALTER OPERATOR FAMILY my_opf1 USING gist RENAME TO my_opf2;
ALTER OPERATOR FAMILY my_opf1 USING gin RENAME TO my_opf2;
ALTER OPERATOR FAMILY my_opf1 USING btree RENAME TO my_opf2;
-- same for operator classes
ALTER OPERATOR FAMILY my_opc1 USING gist RENAME TO my_opc2;
ALTER OPERATOR FAMILY my_opc1 USING gin RENAME TO my_opc2;
ALTER OPERATOR FAMILY my_opc1 USING btree RENAME TO my_opc2;
ALTER OPERATOR CLASS my_opc1 USING gist RENAME TO my_opc2;
ALTER OPERATOR CLASS my_opc1 USING gin RENAME TO my_opc2;
ALTER OPERATOR CLASS my_opc1 USING btree RENAME TO my_opc2;
