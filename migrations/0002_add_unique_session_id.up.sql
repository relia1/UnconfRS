-- Add up migration script here
ALTER TABLE timeslot_assignments
ADD CONSTRAINT unique_session_id UNIQUE (session_id);
