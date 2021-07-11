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
CREATE TABLE options_1(id integer);
ALTER TABLE options_1 SET (autovacuum_enabled = off, fillfactor = 90, parallel_workers = 4);
CREATE TABLE options_2(id integer);
ALTER TABLE options_2 SET (toast_tuple_target = 8000);
CREATE VIEW v1 AS select 2;
CREATE TABLE tbl_rewrite(id integer);
CREATE RULE r1 AS ON INSERT TO tbl_rewrite DO ALSO DELETE FROM v1;
CREATE RULE r2 AS ON INSERT TO tbl_rewrite WHERE id = 0 DO NOTHING;
CREATE RULE r4 AS ON DELETE TO tbl_rewrite DO NOTHING;
CREATE TABLE tbl_trig(id integer);
CREATE FUNCTION ftrig1() RETURNS trigger AS $$ BEGIN END; $$ LANGUAGE plpgsql;
CREATE TRIGGER trig1 AFTER INSERT OR UPDATE ON tbl_trig FOR EACH STATEMENT EXECUTE FUNCTION ftrig1();
CREATE FUNCTION ftrig3() RETURNS trigger AS $$ BEGIN END; $$ LANGUAGE plpgsql;
CREATE TRIGGER trig2 BEFORE INSERT ON tbl_trig FOR EACH ROW EXECUTE FUNCTION ftrig3();
CREATE FUNCTION func_1(integer) RETURNS setof bool AS $$ ; $$ language sql;
CREATE FUNCTION func_2(integer, integer) RETURNS integer AS $$ ; $$ language plpgsql;
CREATE FUNCTION func_3(integer) RETURNS void AS $$
    BEGIN
        RAISE NOTICE 'some message';
        -- some comment
        PERFORM pg_sleep(1);
        RAISE NOTICE 'some other message';
    END;
$$ LANGUAGE plpgsql;
CREATE PROCEDURE func_4()
BEGIN ATOMIC
    SELECT 2;
    SELECT false;
END;
