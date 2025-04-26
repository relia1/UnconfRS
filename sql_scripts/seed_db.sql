DO
$$
    BEGIN
        FOR room_num IN 1..4
            LOOP
                INSERT INTO rooms (name, location, available_spots)
                VALUES (format('room %s', room_num), format('loc %s', room_num), 0);
            END LOOP;
        FOR timeslot in 1..5
            LOOP
                INSERT INTO time_slots (start_time, end_time, duration)
                VALUES (format('0%s:00:00', timeslot)::time, format('0%s:00:00', timeslot + 1)::time, '01:00:00');
            END LOOP;

        FOR room_num in 1..4
            LOOP
                FOR timeslot in 1..5
                    LOOP
                        WITH new_speaker AS (
                            INSERT INTO speakers (name, email, phone_number) VALUES (format('name %s %s', room_num, timeslot),
                                                                                     format('email%s%s@gmail.com', room_num, timeslot),
                                                                                     format('%s%s', room_num, REPEAT(format('%s', timeslot), 9))) RETURNING id)
                        INSERT
                        INTO sessions (speaker_id, title, content, votes)
                        SELECT id,
                               format('title %s %s', room_num, timeslot),
                               format('content %s %s', room_num, timeslot),
                               0
                        FROM new_speaker;
                    END LOOP;
            END LOOP;
    END;
$$;
