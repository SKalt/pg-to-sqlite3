postgres data types https://www.postgresql.org/docs/current/datatype.html#DATATYPE-TABLE
| | name                       | aliases        | description                              | 
|-|:--------------------------:|:--------------:|------------------------------------------|
| | bigint                     | int8           |signed eight-byte integer                 |
|X| bigserial                  | serial8        | autoincrementing eight-byte integer      |
|X| bit [ (n) ]                |                |fixed-length bit string                   |
| | bit varying [ (n) ]        | varbit [ (n) ] | variable-length bit string               |
| | boolean                    | bool logical   | Boolean (true/false)                     |
| | box                        |                | rectangular box on a plane               |
| | bytea                      |                | binary data (“byte array”)               |
| | character [ (n) ]          | char [ (n) ]   | fixed-length character string            |
| | character varying [ (n) ]  | varchar [ (n) ]| variable-length character string         |
| | cidr                       |                | IPv4 or IPv6 network address             |
| | circle                     |                | circle on a plane                        |
| | date                       |                | calendar date (year, month, day)         |
| | double precision           | float8         | double precision floating-point number (8 bytes)|
| | inet                       |                | IPv4 or IPv6 host address                |
| | integer                    | int, int4      | signed four-byte integer                 |
| | interval [ fields ] [ (p) ]|                | time span                                |
| | json                       |                | textual JSON data                        |
| | jsonb                      |                | binary JSON data, decomposed             |
| | line                       |                | infinite line on a plane                 |
| | lseg                       |                | line segment on a plane                  |
| | macaddr                    |                | MAC (Media Access Control) address       |
| | macaddr8                   |                | MAC (Media Access Control) address (EUI-64 format)|
| | money                      |                | currency amount                          |
| | numeric [ (p, s) ]         | decimal [ (p, s) ] | exact numeric of selectable precision|
| | path                       |                |geometric path on a plane                 |
| | pg_lsn                     |                |PostgreSQL Log Sequence Number            |
| | pg_snapshot                |                |user-level transaction ID snapshot        |
| | point                      |                |geometric point on a plane                |
| | polygon                    |                |closed geometric path on a plane          |
| | real                       | float4         |single precision floating-point number (4 bytes)|
| | smallint                   | int2           |signed two-byte integer                   |
| | smallserial                | serial2        |autoincrementing two-byte integer         |
| | serial                     | serial4        |autoincrementing four-byte integer        |
| | text                       |                |variable-length character string          |
| | tsquery                    |                |text search query                         |
| | tsvector                   |                |text search document                      |
| | txid_snapshot              |                | user-level transaction ID snapshot (deprecated; see pg_snapshot) |
| | uuid                       |                | universally unique identifier            |
| | xml                        |                | XML data                                 |
| | timestamp [ (p) ] with time zone        | timestamptz | date and time, including time zone|
| | time      [ (p) ] with time zone        | timetz      | time of day, including time zone|
| | timestamp [ (p) ] [ without time zone ] |             | date and time (no time zone)|
| | time      [ (p) ] [ without time zone ] |             | time of day (no time zone)|

sqlite3 storage classes: https://www.sqlite.org/datatype3.html
- NUMERIC
- INTEGER
- REAL
- TEXT
- BLOB
- NULL

> 
    1. If the declared type contains the string "INT" then it is assigned INTEGER affinity.
    2. If the declared type of the column contains any of the strings "CHAR", "CLOB", or "TEXT" then that column has TEXT affinity.    Notice that the type VARCHAR contains the string "CHAR" and is thus assigned TEXT affinity.
    3. If the declared type for a column contains the string "BLOB" or if no type is specified then the column has affinity BLOB.
    4. If the declared type for a column contains any of the strings "REAL", "FLOA", or "DOUB" then the column has REAL affinity.
    5. Otherwise, the affinity is NUMERIC.
25
sqlite recognized datatypes:
| name              | Affinity | Rule Used To Determine Affinity |
| INT               | INTEGER  | 1 |  
| INTEGER           | INTEGER  | 1 | 
| TINYINT           | INTEGER  | 1 | 
| SMALLINT          | INTEGER  | 1 | 
| MEDIUMINT         | INTEGER  | 1 | 
| BIGINT            | INTEGER  | 1 | 
| UNSIGNED BIG INT  | INTEGER  | 1 | 
| INT2              | INTEGER  | 1 | 
| INT8              | INTEGER  | 1 |
| CHARACTER         | TEXT     | 2 |
| VARCHAR           | TEXT     | 2 |
| VARYING CHARACTER | TEXT     | 2 |
| NCHAR             | TEXT     | 2 |
| NATIVE CHARACTER  | TEXT     | 2 |
| NVARCHAR          | TEXT     | 2 |
| TEXT              | TEXT     | 2 |
| CLOB              | TEXT     | 2 |
| BLOB              | BLOB     | 3 |
| REAL              | REAL     | 4 |
| DOUBLE            | REAL     | 4 |
| DOUBLE PRECISION  | REAL     | 4 |
| FLOAT             | REAL     | 4 |
| NUMERIC           | NUMERIC  | 5 |
| DECIMAL           | NUMERIC  | 5 |
| BOOLEAN           | NUMERIC  | 5 |
| DATE              | NUMERIC  | 5 |
| DATETIME          | NUMERIC  | 5 |
