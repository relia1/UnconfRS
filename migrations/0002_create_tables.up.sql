CREATE TABLE time_slots (
	id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    duration INTERVAL NOT NULL,
    schedule_id INTEGER,
    speaker_id INTEGER,
    topic_id INTEGER REFERENCES topics(id)
);
