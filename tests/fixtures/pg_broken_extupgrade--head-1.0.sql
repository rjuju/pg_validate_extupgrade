-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "CREATE EXTENSION pg_broken_extupgrade" to load this file. \quit

CREATE TABLE dump_0(id integer);
SELECT pg_extension_config_dump('dump_0', '');
CREATE TABLE dump_1(id integer);
SELECT pg_extension_config_dump('dump_1', 'WHERE id != 0');
CREATE TABLE tbl0(id integer, val text);
CREATE STATISTICS tbl0_stats ON id, val FROM tbl0;
COMMENT ON STATISTICS tbl0_stats IS 'statistics without qual';
CREATE STATISTICS tbl0_stats_n (ndistinct) ON (id + 1), val FROM tbl0;
CREATE INDEX ON tbl0 (id);
COMMENT ON INDEX tbl0_id_idx IS 'index without qual';
CREATE TABLE tbl1(id integer, val text);
CREATE TABLE tbl2(id integer, val text);
CREATE TABLE tbl3(id integer, val text);
CREATE TABLE tbl4(id integer, val varchar(20));
CREATE TABLE logged(id integer, val text);
COMMENT ON TABLE logged IS 'I''m logged';
COMMENT ON COLUMN logged.id IS 'id column';
CREATE TABLE missing2(id integer);
CREATE TABLE papart();
CREATE TABLE main(id integer);
CREATE TABLE ref(id integer);
CREATE TABLE main2(id integer primary key, val text CHECK (length(val) > 1));
COMMENT ON CONSTRAINT main2_pkey ON main2 IS 'more than 1';
CREATE TABLE ref2(id integer references main2 (id) ON UPDATE no action ON DELETE restrict);
CREATE TABLE options_1(id integer);
ALTER TABLE options_1 SET (fillfactor = 80, toast_tuple_target = 8100);
CREATE TABLE options_2(id integer);
ALTER TABLE options_2 SET (fillfactor = 80, toast_tuple_target = 8100);
CREATE VIEW v1 AS select 1;
COMMENT ON VIEW v1 IS 'one';
CREATE TABLE tbl_rewrite(id integer);
CREATE RULE r1 AS ON UPDATE TO tbl_rewrite DO ALSO DELETE FROM v1;
CREATE RULE r2 AS ON INSERT TO tbl_rewrite DO NOTHING;
COMMENT ON RULE r2 ON tbl_rewrite IS 'no qual';
CREATE RULE r3 AS ON DELETE TO tbl_rewrite DO NOTHING;
CREATE TABLE tbl_trig(id integer);
CREATE FUNCTION ftrig1() RETURNS trigger AS $$ BEGIN END; $$ LANGUAGE plpgsql;
CREATE TRIGGER trig1 AFTER INSERT OR UPDATE ON tbl_trig FOR EACH ROW EXECUTE FUNCTION ftrig1();
CREATE FUNCTION ftrig2() RETURNS trigger AS $$ BEGIN END; $$ LANGUAGE plpgsql;
CREATE TRIGGER trig2 BEFORE INSERT ON tbl_trig FOR EACH ROW EXECUTE FUNCTION ftrig2();
CREATE FUNCTION func_1(integer) RETURNS void AS $$ ; $$ language plpgsql;
COMMENT ON FUNCTION func_1(integer) IS 'plpgsql';
CREATE FUNCTION func_2(integer = 1, out integer) RETURNS integer AS $$ ; $$ language plpgsql;
CREATE FUNCTION func_3(smallint) RETURNS void AS $$
    BEGIN
        RAISE NOTICE 'some message';
        PERFORM pg_sleep(1);
        RAISE NOTICE 'some other message';
    END;
$$ LANGUAGE plpgsql;
CREATE FUNCTION func_3b(smallint) RETURNS void AS $$;$$ language plpgsql;
CREATE FUNCTION func_4() RETURNS bool
BEGIN ATOMIC
    SELECT 1;
    SELECT false;
END;
CREATE OR REPLACE FUNCTION fct_evt_trigger_1()
RETURNS event_trigger
LANGUAGE plpgsql
AS $_$
DECLARE
BEGIN
END; $_$;
CREATE EVENT TRIGGER evt_trigger_1
    ON ddl_command_end
    WHEN tag IN ('CREATE EXTENSION', 'CREATE TABLE')
    EXECUTE PROCEDURE fct_evt_trigger_1() ;
COMMENT ON EVENT TRIGGER evt_trigger_1 IS 'ext and table';
CREATE EVENT TRIGGER evt_trigger_2
    ON ddl_command_end
    WHEN tag IN ('DROP EXTENSION')
    EXECUTE PROCEDURE fct_evt_trigger_1() ;
CREATE OPERATOR >< (
    PROCEDURE = func_1,
    rightarg = int4
);
CREATE OPERATOR >< (
    PROCEDURE = func_3,
    rightarg = int2
);
CREATE TYPE shell_1;
CREATE TYPE typ_composite AS (col1 integer, col2 text, col3 text collate "C");
CREATE TYPE typ_enum AS ENUM('a', 'b');
CREATE TYPE typ_range AS RANGE(SUBTYPE = int4);
CREATE FUNCTION fcast_i_p(integer) returns point as $$;$$ language plpgsql;
CREATE FUNCTION fcast_i_p1(integer) returns point as $$;$$ language plpgsql;
CREATE CAST (integer AS point) WITH FUNCTION fcast_i_p(int) AS ASSIGNMENT;
CREATE AGGREGATE agg_1 (int) (sfunc = int4smaller, stype = int4);
