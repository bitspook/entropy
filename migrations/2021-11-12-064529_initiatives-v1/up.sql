CREATE TABLE initiatives (
  slug VARCHAR(100) PRIMARY KEY,
  title VARCHAR(400) NOT NULL,
  description VARCHAR,
  SOURCE VARCHAR(20),
  desc_format VARCHAR(10) NOT NULL DEFAULT 'md',
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON initiatives FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();


CREATE TABLE goals (
  slug VARCHAR(100) PRIMARY KEY,
  title VARCHAR(400) NOT NULL,
  description VARCHAR,
  desc_format VARCHAR(10) NOT NULL DEFAULT 'md',
  initiative_slug VARCHAR REFERENCES initiatives ON DELETE CASCADE NOT NULL,
  target INTEGER NOT NULL,
  iteration_interval INTERVAL,
  last_iteration_at TIMESTAMP DEFAULT NOW() NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON goals FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE activity(
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  activity_type VARCHAR(100) NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  meta JSONB NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON activity FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();


CREATE TABLE goal_ledger (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  goal_slug VARCHAR(100) REFERENCES goals,
  xp INTEGER NOT NULL DEFAULT 1,
  event_type VARCHAR(100) NOT NULL,
  display VARCHAR(200),
  meta JSONB NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON goal_ledger FOR EACH ROW EXECUTE
PROCEDURE trigger_set_timestamp();
