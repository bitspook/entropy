CREATE EXTENSION "uuid-ossp";


CREATE FUNCTION trigger_set_timestamp()
  RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TABLE tags (
  name VARCHAR(40) PRIMARY KEY,
  display_name VARCHAR(40) UNIQUE,
  icon VARCHAR,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON tags FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE groups (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name VARCHAR(200) UNIQUE NOT NULL,
  slug VARCHAR(200) UNIQUE NOT NULL,
  description VARCHAR,
  desc_format VARCHAR(10) DEFAULT 'md' NOT NULL,
  photos TEXT ARRAY NOT NULL DEFAULT ARRAY[]::VARCHAR[],
  source VARCHAR(60),
  source_link VARCHAR(200),
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON groups FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE group_tags (
  group_id UUID REFERENCES groups NOT NULL,
  tag_name VARCHAR REFERENCES tags NOT NULL,
  PRIMARY KEY (group_id, tag_name),
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON group_tags FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE organizations (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name VARCHAR(200) NOT NULL,
  description VARCHAR,
  website VARCHAR(200),
  logo VARCHAR,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON organizations FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();

CREATE TABLE venues (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  address VARCHAR(400) NOT NULL,
  directions VARCHAR(400),
  organization_id UUID REFERENCES organizations,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON venues FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE events (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  title VARCHAR(200) NOT NULL,
  slug VARCHAR(200) UNIQUE NOT NULL,
  description VARCHAR,
  desc_format VARCHAR(10) DEFAULT 'md' NOT NULL,
  group_id UUID REFERENCES groups NOT NULL,
  venue_id UUID REFERENCES venues,
  photos TEXT ARRAY NOT NULL DEFAULT ARRAY[]::VARCHAR[],
  source VARCHAR(60),           -- source from which the event is obtained; e.g meetup.com
  source_link VARCHAR(200),
  start_time TIMESTAMP NOT NULL,
  end_time TIMESTAMP NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON events FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE event_tags (
  event_id UUID REFERENCES events,
  tag_name VARCHAR REFERENCES tags,
  PRIMARY KEY (event_id, tag_name),
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON event_tags FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();
