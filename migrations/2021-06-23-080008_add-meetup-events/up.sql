CREATE TABLE meetup_events (
  id VARCHAR PRIMARY KEY NOT NULL,
  slug VARCHAR UNIQUE NOT NULL,
  group_slug VARCHAR NOT NULL,
  title VARCHAR NOT NULL,
  description VARCHAR,
  start_time DATETIME NOT NULL,
  end_time DATETIME NOT NULL,
  is_online BOOLEAN NOT NULL DEFAULT FALSE,
  charges DOUBLE,
  currency VARCHAR,
  link VARCHAR NOT NULL,
  venue VARCHAR,
  FOREIGN KEY(group_slug) REFERENCES meetup_groups(slug)
);
