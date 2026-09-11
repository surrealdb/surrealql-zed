-- Fixture for scripts/check-queries.py.
--
-- Every construct `languages/surql/*.scm` claims to match should appear at
-- least once here, so a query that stops matching shows up as a changed
-- capture count rather than staying green. This file must parse with zero
-- ERROR and MISSING nodes against the grammar revision pinned in
-- extension.toml; a construct the pinned grammar cannot parse does not
-- belong here.
--
-- After editing, run scripts/update-query-baseline.sh and commit the result.

/* A block comment,
   spanning lines. */

DEFINE NAMESPACE test;
DEFINE DATABASE app;
DEFINE TABLE person SCHEMAFULL PERMISSIONS FOR select, update WHERE published = true;
DEFINE FIELD name.first ON person TYPE string ASSERT $value != NONE;
DEFINE FIELD tags ON person TYPE array<string, 10> DEFAULT [];
DEFINE FIELD score ON person TYPE option<int | float> VALUE 0;
DEFINE INDEX idx_name ON person FIELDS name UNIQUE;
DEFINE INDEX idx_vec ON person FIELDS embedding HNSW DIMENSION 128 DIST COSINE TYPE F32;
DEFINE ANALYZER ascii TOKENIZERS class FILTERS lowercase, ascii, edgengram(2, 10);
DEFINE EVENT publish ON TABLE person WHEN $before.published = false THEN {
    UPDATE audit SET at = time::now();
};
DEFINE PARAM $rate VALUE 1.5;
DEFINE FUNCTION fn::greet($name: string, $times: int) {
    RETURN function($name) { return `Hello, ${arguments[0]}!`; };
};
DEFINE USER admin ON ROOT PASSWORD 'secret' ROLES OWNER;

ALTER TABLE person COMMENT 'people';
REMOVE INDEX idx_name ON person;
REMOVE FUNCTION fn::greet;

LET $id = person:tobie;
LET $range = person:1..4;
LET $re = /^a.*z$/;
LET $msg = $"Hello {$id}, you have {$rate} credits";
LET $dur = 1h30m15s;
LET $big = 102023.1dec;
LET $f = 9.7e-7;
LET $obj = { name: 'Tobie', age: 29, nested: { ok: true, missing: NONE } };
LET $double = |$x: int| $x * 2;

SELECT * FROM person WHERE age > 18 AND name.first ?= 'Tobie' LIMIT 10;
SELECT ->likes->person.* AS friends FROM user:one FETCH friends;
SELECT person.spouse.?.name, count() FROM person GROUP ALL;
SELECT * FROM |test:1..4| WHERE id INSIDE $range;
SELECT * FROM person WHERE embedding <|2,40|> [0.1, 0.2];

CREATE person SET name.first = 'John', created = time::now();
UPDATE person:tobie SET settings.theme = 'dark' RETURN AFTER;
UPDATE person:tobie UNSET name, email, age;
DELETE person WHERE age < 0;
RELATE user:one->likes->person:two SET at = time::now();
INSERT INTO person (name, age) VALUES ('Jaime', 35);

SHOW CHANGES FOR TABLE person SINCE 1 LIMIT 10;
INFO FOR TABLE person;
RETURN 8 % 3;
RETURN -$rate;
