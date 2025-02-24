BEGIN;

DELETE
FROM timeslot_assignments;
DELETE
FROM time_slots;
DELETE
FROM topics;
DELETE
FROM speakers;
DELETE
FROM rooms;
DELETE
FROM users;

COMMIT;
