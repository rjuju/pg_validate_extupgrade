pg_validate_extupgrade
======================

Tool to validate PostgreSQL extension upgrade script.

Usage
-----

```
USAGE:
    pg_validate_extupgrade [OPTIONS] --extname <extname> --from <from> --to <to>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --dbname <dbname>      database name
    -e, --extname <extname>    extension to test
        --from <from>          initial version of the extension
    -h, --host <host>          database server host or socket directory
    -p, --port <port>          database server port
        --to <to>              upgraded version of the extension
    -U, --user <user>          database user name
```

Note that the connection parameters default follow the same rules as PostgreSQL
official client.

Example
-------

```
$ pg_validate_extupgrade -e pg_broken_extupgrade --from head-1.0 --to head-1.1
Connected, server version 140000
ERROR: Differences found:
- mismatch found for Extension pg_broken_extupgrade:
  - in relations:
    installed and upgraded both have 14 Relation but some mismatch in them:
      3 Relation missing in installed:
        - public.missing2
        - public.papart
        - public.tbl1

      3 Relation missing in upgraded:
        - public.missing1
        - public.broken_tbl1
        - public.tbl3_id_seq

      - mismatch found for Relation public.options_1:
        - in reloptions:
          installed has 1 more Option (3) than upgraded (2)
            1 Option missing in installed:
              - toast_tuple_target

            2 Option missing in upgraded:
              - parallel_workers
              - autovacuum_enabled

            - mismatch found for fillfactor:
              - 90
              + 80

      - mismatch found for Relation public.options_2:
        - in reloptions:
          upgraded has 1 more Option (2) than installed (1)
            1 Option missing in installed:
              - fillfactor

            - mismatch found for toast_tuple_target:
              - 8000
              + 8100

      - mismatch found for Relation public.tbl4:
        - in attributes:
          - mismatch for elem #1:
            - mismatch found for Attribute val:
              - in atttype:
                - character varying(30)
                + character varying(20)

      - mismatch found for Relation public.tbl0:
        - in indexes:
          installed and upgraded both have 1 Index but some mismatch in them:
            - mismatch found for Index public.tbl0_id_idx:
              - in inddef:
                - CREATE INDEX tbl0_id_idx ON public.tbl0 USING btree (id) WHERE (id > 0)
                + CREATE INDEX tbl0_id_idx ON public.tbl0 USING btree (id)

        - in stats:
          installed and upgraded both have 2 ExtendedStatistic but some mismatch in them:
            - mismatch found for ExtendedStatistic public.tbl0_stats_n:
              - in stxkind:
                - mismatch for elem #0:
                  - f
                  + d

            - mismatch found for ExtendedStatistic public.tbl0_stats:
              - in columns:
                - id, val, ((id * 2))
                + id, val

              - in stxkind:
                - installed has 1 more elements (4) than upgraded (3)
                - mismatch for elem #3:
                  - upgraded has no value, while installed has
                    + 101

      - mismatch found for Relation public.main:
        - in attributes:
          - mismatch for elem #0:
            - mismatch found for Attribute id:
              - in attnotnull:
                - true
                + false

        - in indexes:
          installed has 1 more Index (1) than upgraded (0)
            1 Index missing in upgraded:
              - public.main_pkey

        - in constraints:
          installed has 1 more Constraint (1) than upgraded (0)
            1 Constraint missing in upgraded:
              - public.main_pkey

        - in relhasindex:
          - true
          + false

        - in relhastriggers:
          - true
          + false

      - mismatch found for Relation public.tbl2:
        - in attributes:
          - mismatch for elem #0:
            - mismatch found for Attribute broken_id:
              - in attname:
                - broken_id
                + id

          - mismatch for elem #1:
            - mismatch found for Attribute val:
              - in attcollation:
                - upgraded has no value, while installed has
                  + C

      - mismatch found for Relation public.ref2:
        - in constraints:
          installed and upgraded both have 1 Constraint but some mismatch in them:
            - mismatch found for Constraint public.ref2_id_fkey:
              - in condef:
                - FOREIGN KEY (id) REFERENCES public.main2(id) ON UPDATE CASCADE ON DELETE CASCADE
                + FOREIGN KEY (id) REFERENCES public.main2(id) ON DELETE RESTRICT

      - mismatch found for Relation public.ref:
        - in constraints:
          installed has 1 more Constraint (1) than upgraded (0)
            1 Constraint missing in upgraded:
              - public.ref_id_fkey

        - in relhastriggers:
          - true
          + false

      - mismatch found for Relation public.logged:
        - in relpersistence:
          - u
          + p

        - in relacl:
          - upgraded has no value, while installed has
            + {rjuju=arwdDxt/rjuju}

        - in reloptions:
          - upgraded has no value, while installed has
            + fillfactor=80

      - mismatch found for Relation public.main2:
        - in constraints:
          installed and upgraded both have 2 Constraint but some mismatch in them:
            - mismatch found for Constraint public.main2_val_check:
              - in condef:
                - CHECK ((length(val) > 2))
                + CHECK ((length(val) > 1))

      - mismatch found for Relation public.tbl3:
        - in attributes:
          - mismatch for elem #0:
            - mismatch found for Attribute id:
              - in attnotnull:
                - true
                + false

              - in attdefault:
                - upgraded has no value, while installed has
                  + nextval('public.tbl3_id_seq'::regclass)

          - mismatch for elem #1:
            - mismatch found for Attribute val:
              - in atttype:
                - character varying(30)
                + text

        - in relam:
          - installed has no value, while upgraded has
            + heap

        - in relkind:
          - p
          + r

        - in relpartkey:
          - upgraded has no value, while installed has
            + LIST (id)

      - mismatch found for Relation public.tbl_rewrite:
        - in rules:
          installed and upgraded both have 3 Rule but some mismatch in them:
            1 Rule missing in installed:
              - r3

            1 Rule missing in upgraded:
              - r4

            - mismatch found for Rule r1:
              - in inddef:
                - CREATE RULE r1 AS
    ON INSERT TO public.tbl_rewrite DO  DELETE FROM public.v1;
                + CREATE RULE r1 AS
    ON UPDATE TO public.tbl_rewrite DO  DELETE FROM public.v1;

            - mismatch found for Rule r2:
              - in inddef:
                - CREATE RULE r2 AS
    ON INSERT TO public.tbl_rewrite
   WHERE (new.id = 0) DO NOTHING;
                + CREATE RULE r2 AS
    ON INSERT TO public.tbl_rewrite DO NOTHING;

      - mismatch found for Relation public.v1:
        - in rules:
          installed and upgraded both have 1 Rule but some mismatch in them:
            - mismatch found for Rule _RETURN:
              - in inddef:
                - CREATE RULE "_RETURN" AS
    ON SELECT TO public.v1 DO INSTEAD  SELECT 2;
                + CREATE RULE "_RETURN" AS
    ON SELECT TO public.v1 DO INSTEAD  SELECT 1;

      - mismatch found for Relation public.tbl_trig:
        - in triggers:
          installed and upgraded both have 2 Trigger but some mismatch in them:
            - mismatch found for Trigger trig1:
              - in tgdef:
                - CREATE TRIGGER trig1 AFTER INSERT OR UPDATE ON public.tbl_trig FOR EACH STATEMENT EXECUTE FUNCTION public.ftrig1()
                + CREATE TRIGGER trig1 AFTER INSERT OR UPDATE ON public.tbl_trig FOR EACH ROW EXECUTE FUNCTION public.ftrig1()

            - mismatch found for Trigger trig2:
              - in tgdef:
                - CREATE TRIGGER trig2 BEFORE INSERT ON public.tbl_trig FOR EACH ROW EXECUTE FUNCTION public.ftrig3()
                + CREATE TRIGGER trig2 BEFORE INSERT ON public.tbl_trig FOR EACH ROW EXECUTE FUNCTION public.ftrig2()

  - in routines:
    installed and upgraded both have 6 Routine but some mismatch in them:
      2 Routine missing in installed:
        - public.func_2(integer DEFAULT 1, OUT integer)
        - public.ftrig2()

      2 Routine missing in upgraded:
        - public.func_2(integer, integer)
        - public.ftrig3()

      - mismatch found for Routine public.func_1(integer):
        - in prolang:
          - sql
          + plpgsql

        - in prorows:
          - 1000
          + 0

        - in prorettype:
          - SETOF boolean
          + void


      - mismatch found for Routine public.func_3(integer):
        - in source:
--- installed
--- upgraded
@@ -1,7 +1,6 @@

     BEGIN
         RAISE NOTICE 'some message';
-        -- some comment
         PERFORM pg_sleep(1);
         RAISE NOTICE 'some other message';
     END;
      - mismatch found for Routine public.func_4():
        - in prokind:
          - p
          + f

        - in prorettype:
          - installed has no value, while upgraded has
            + boolean

        - in source:
--- installed
--- upgraded
@@ -1,4 +1,4 @@
 BEGIN ATOMIC
- SELECT 2;
+ SELECT 1;
  SELECT false AS bool;
 END
```

LICENSE
    Copyright (c) 2021 Julien Rouhaud - All rights reserved.

      This program is free software: you can redistribute it and/or modify
      it under the terms of the GNU General Public License as published by
      the Free Software Foundation, either version 3 of the License, or
      any later version.

      This program is distributed in the hope that it will be useful,
      but WITHOUT ANY WARRANTY; without even the implied warranty of
      MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
      GNU General Public License for more details.

      You should have received a copy of the GNU General Public License
      along with this program.  If not, see < http://www.gnu.org/licenses/ >.
