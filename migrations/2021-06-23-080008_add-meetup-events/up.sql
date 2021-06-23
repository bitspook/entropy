CREATE TABLE meetup_events (
  id VARCHAR PRIMARY KEY NOT NULL,
  created DATETIME NOT NULL,
  updated DATETIME NOT null,
  duration INTEGER,
  name VARCHAR NOT NULL,
  status VARCHAR NOT null,
  time DATETIME NOT NULL,
  local_date VARCHAR NOT NULL,
  local_time VARCHAR NOT NULL,
  utc_offset INTEGER NOT NULL,
  is_online_event BOOLEAN NOT NULL DEFAULT FALSE,
  link VARCHAR NOT NULL,
  description VARCHAR,
  how_to_find_us VARCHAR,
  visibility VARCHAR,
  member_pay_fee BOOLEAN,
  venue_visibility VARCHAR
);
