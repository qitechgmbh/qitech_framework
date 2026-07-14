#!/usr/bin/env fish

set SCRIPT_DIR (dirname (realpath (status filename)))
set SANDBOX "$SCRIPT_DIR/sandbox"

cd $SCRIPT_DIR
clickhouse-server -C config.xml