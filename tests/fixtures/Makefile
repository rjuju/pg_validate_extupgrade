EXTENSION    = pg_broken_extupgrade
DATA         = $(wildcard *--*.sql)

PG_CONFIG    = pg_config
PGXS         = $(shell $(PG_CONFIG) --pgxs)

include $(PGXS)
