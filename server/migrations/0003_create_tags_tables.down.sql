-- Rollback migration
DROP INDEX IF EXISTS tower_sessions.session_expiry_idx;
DROP TABLE IF EXISTS tower_sessions.session;
DROP SCHEMA IF EXISTS tower_sessions;

DROP TABLE IF EXISTS user_votes;
DROP TABLE IF EXISTS timeslot_assignments;
DROP TABLE IF EXISTS users_groups;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS groups_permissions;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS time_slots;
DROP TABLE IF EXISTS rooms;
DROP TABLE IF EXISTS permissions;
DROP TABLE IF EXISTS groups;