BEGIN;

DELETE
FROM timeslot_assignments;
DELETE
FROM time_slots;
DELETE
FROM sessions;
DELETE
FROM rooms;
DELETE
FROM users;

COMMIT;
