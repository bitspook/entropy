-- This file should undo anything in `up.sql`
DROP TABLE event_tags;
DROP TABLE events;
DROP TABLE venues;
DROP TABLE organizations;
DROP TABLE group_tags;
DROP TABLE groups;
DROP TABLE tags;
DROP EXTENSION "uuid-ossp";
DROP FUNCTION trigger_set_timestamp();
