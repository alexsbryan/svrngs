-- Saturday's board. A plan and the things to bring for it are anyone's to
-- change; a claim and a vote are their person's own, since each key begins
-- person_key (docs/guide/database.md, "Whose row it is"). Nothing is deleted:
-- a row people may still edit is taken off with a column, since a delete
-- beats an edit it never saw (docs/guide/database-patterns.md).

-- One plan for the day. `id` is crypto.randomUUID() from the page that added
-- it, `at` the time as <input type="time"> gives it ('14:30'), `added_by`
-- the adder's you_key, and `off` 1 once someone took it off the board.
CREATE TABLE plans (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT,
  at TEXT,
  place TEXT,
  added_by TEXT,
  off INTEGER
) STRICT;

-- A thing to bring for a plan, one line of a pasted list. Its key is the line
-- trimmed and in lower case, so two people pasting "Ice" make one row;
-- `words` is the line as typed, and `off` 1 once someone took it off.
CREATE TABLE things (
  plan TEXT NOT NULL,
  thing TEXT NOT NULL,
  words TEXT,
  off INTEGER,
  PRIMARY KEY (plan, thing)
) STRICT;

-- "I'll bring ice": a row per claimer, so two claims made apart both show.
-- `withdrawn` is 1 once its person took it back.
CREATE TABLE claims (
  person_key TEXT NOT NULL,
  id TEXT NOT NULL,
  plan TEXT,
  what TEXT,
  withdrawn INTEGER,
  PRIMARY KEY (person_key, id)
) STRICT;

-- A row per person per plan; a plan's total counts the rows whose `yes` is 1,
-- and taking a vote back writes 0.
CREATE TABLE votes (
  person_key TEXT NOT NULL,
  plan TEXT NOT NULL,
  yes INTEGER,
  PRIMARY KEY (person_key, plan)
) STRICT;
