CREATE TABLE time_slots (
	id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    duration INTEGER NOT NULL,
    schedule_id INTEGER NOT NULL DEFAULT 1,
    speaker_id INTEGER NOT NULL,
    topic_id INTEGER REFERENCES topics(id)
);
