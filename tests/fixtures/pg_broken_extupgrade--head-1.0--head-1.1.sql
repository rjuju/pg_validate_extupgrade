-- This program is open source, licensed under the PostgreSQL License.
-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "ALTER EXTENSION pg_broken_extupgrade UPDATE" to load this file. \quit
SET work_mem = '321kB';
SET LOCAL maintenance_work_mem TO '6666';
SET LOCAL client_min_messages = WARNING;
