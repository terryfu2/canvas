CREATE TABLE canvas (
  x integer,
  y integer,
  colour integer NOT NULL,
  updated integer DEFAULT 0,
  PRIMARY KEY(x, y)
);
