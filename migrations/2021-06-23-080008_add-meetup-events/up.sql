CREATE TABLE meetup_events (
  id SERIAL PRIMARY KEY,
  slug VARCHAR UNIQUE NOT NULL,
  group_slug VARCHAR NOT NULL REFERENCES meetup_groups(slug) ON UPDATE CASCADE,
  title VARCHAR NOT NULL,
  description VARCHAR,
  start_time TIMESTAMP NOT NULL,
  end_time TIMESTAMP NOT NULL,
  is_online BOOLEAN NOT NULL DEFAULT FALSE,
  charges FLOAT,
  currency VARCHAR,
  link VARCHAR NOT NULL,
  venue VARCHAR
);
