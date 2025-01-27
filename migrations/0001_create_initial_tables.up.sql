CREATE TABLE topics (
	id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    speaker_id INT NOT NULL,
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

CREATE TABLE speakers (
	id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone_number TEXT NOT NULL
);

CREATE TABLE time_slots (
	id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    duration INTERVAL NOT NULL
);

CREATE TABLE timeslot_assignments
(
    id           INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    time_slot_id INTEGER REFERENCES time_slots (id),
    speaker_id   INTEGER REFERENCES speakers (id),
    topic_id INTEGER REFERENCES topics(id),
    room_id      INTEGER REFERENCES rooms (id),
    UNIQUE (time_slot_id, room_id)
);

CREATE TABLE users (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL
);