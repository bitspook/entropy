-- This file should undo anything in `up.sql`
DROP EXTENSION "uuid-ossp";
DROP FUNCTION trigger_set_timestamp();
DROP TABLE group_tags;
DROP TABLE event_tags;
DROP TABLE events;
DROP TABLE groups;
DROP TABLE tags;
DROP TABLE venues;
