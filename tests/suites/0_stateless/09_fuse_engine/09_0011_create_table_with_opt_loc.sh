#!/usr/bin/env bash

CURDIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
. "$CURDIR"/../../../shell_env.sh


## create table t09_0011
echo "create table t09_0011(c int)" | $MYSQL_CLIENT_CONNECT
echo "insert into t09_0011 values(1)" | $MYSQL_CLIENT_CONNECT
echo "show value of table being cloned"
echo "select *  from t09_0011" | $MYSQL_CLIENT_CONNECT

## get the snapshot id
SNAPSHOT_ID=$(echo "select snapshot_id  from fuse_history('default', 't09_0011')" | $MYSQL_CLIENT_CONNECT)

## create a shallow clones of t09_0011 by using the table option 'snapshot_loc'
## using lower case option key
echo "create table t09_0011_clone1(c int) snapshot_loc='_ss/$SNAPSHOT_ID'" | $MYSQL_CLIENT_CONNECT
## using upper case option key
echo "create table t09_0011_clone2(c int) SNAPSHOT_LOC='_ss/$SNAPSHOT_ID'" | $MYSQL_CLIENT_CONNECT
echo "checking table clone (lower option option key)"
echo "select *  from t09_0011_clone1" | $MYSQL_CLIENT_CONNECT
echo "checking table clone (upper case option key)"
echo "select *  from t09_0011_clone2" | $MYSQL_CLIENT_CONNECT

## Drop table.
echo "drop table  t09_0011" | $MYSQL_CLIENT_CONNECT
echo "drop table  t09_0011_clone2" | $MYSQL_CLIENT_CONNECT
echo "drop table  t09_0011_clone1" | $MYSQL_CLIENT_CONNECT
