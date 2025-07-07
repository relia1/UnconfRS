-- Add down migration script here
ALTER TABLE timeslot_assignments
DROP CONSTRAINT unique_session_id;
