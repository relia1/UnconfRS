CREATE TABLE users
(
    id       INTEGER GENERATED ALWAYS AS IDENTITY UNIQUE PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password TEXT        NOT NULL
);

CREATE TABLE user_info
(
    id           INTEGER GENERATED ALWAYS AS IDENTITY UNIQUE PRIMARY KEY,
    user_id      INTEGER REFERENCES users (id) UNIQUE,
    name         TEXT NOT NULL,
    email        TEXT,
    phone_number TEXT
);

CREATE TABLE groups
(
    id   INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

INSERT INTO groups (name)
VALUES ('user');
INSERT INTO groups (name)
VALUES ('facilitator');
INSERT INTO groups (name)
VALUES ('admin');

CREATE TABLE permissions
(
    id   INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

INSERT INTO permissions (name)
values ('default');
INSERT INTO permissions (name)
values ('staff');
INSERT INTO permissions (name)
values ('superuser');

CREATE TABLE users_groups
(
    user_id  INTEGER REFERENCES users (id),
    group_id INTEGER REFERENCES groups (id),
    PRIMARY KEY (user_id, group_id)
);

CREATE TABLE groups_permissions
(
    group_id      INTEGER REFERENCES groups (id),
    permission_id INTEGER REFERENCES permissions (id),
    PRIMARY KEY (group_id, permission_id)
);

INSERT INTO groups_permissions (group_id, permission_id)
values ((SELECT id FROM groups WHERE name = 'user'),
        (SELECT id FROM permissions WHERE name = 'default')),
       ((SELECT id FROM groups WHERE name = 'facilitator'),
        (SELECT id FROM permissions WHERE name = 'staff')),
       ((SELECT id FROM groups WHERE name = 'admin'),
        (SELECT id FROM permissions WHERE name = 'superuser'));

CREATE SCHEMA tower_sessions;

CREATE TABLE tower_sessions.session
(
    id          TEXT PRIMARY KEY,
    data        BYTEA       NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

CREATE INDEX session_expiry_idx ON tower_sessions.session (expiry_date);

CREATE TABLE topics (
	id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INTEGER REFERENCES user_info (user_id) NOT NULL,
	title TEXT NOT NULL,
	content TEXT NOT NULL,
    votes INT NOT NULL
);

CREATE TABLE rooms (
	id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name TEXT NOT NULL,
	location TEXT NOT NULL,
    available_spots INT NOT NULL
);

CREATE TABLE time_slots (
	id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    duration INTERVAL NOT NULL
);

CREATE TABLE timeslot_assignments
(
    id      INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    time_slot_id INTEGER REFERENCES time_slots (id),
    creator INTEGER REFERENCES users (id),
    topic_id INTEGER REFERENCES topics(id),
    room_id INTEGER REFERENCES rooms (id),
    UNIQUE (time_slot_id, room_id)
);


