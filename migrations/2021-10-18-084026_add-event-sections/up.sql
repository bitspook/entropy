CREATE TABLE event_sections (
  name VARCHAR(200) NOT NULL,
  title VARCHAR(400) NOT NULL,
  logo VARCHAR(200),
  description VARCHAR,
  desc_format VARCHAR(20) NOT NULL DEFAULT 'md',
  start_time TIMESTAMP NOT NULL,
  end_time TIMESTAMP NOT NULL,
  event_id UUID REFERENCES events ON DELETE CASCADE NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL,
  PRIMARY KEY (name, event_id)
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON event_sections FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();
