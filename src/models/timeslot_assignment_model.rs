use crate::models::room_model::Room;
use crate::models::timeslot_model::{TimeSlot, TimeslotAssignment, TimeslotAssignmentForm, TimeslotRequest};
use crate::models::topics_model::Topic;
use chrono::NaiveTime;
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::error::Error;
use tracing::trace;

/// Assigns topics to timeslots.
///
/// This function assigns topics to timeslots based on the provided topics, rooms, and existing
/// timeslots. The topics are assigned to the timeslots in the order they are provided, starting
/// with the first topic and moving to the next topic for each room.
///
/// # Parameters
/// - `topics`: A slice of `Topic` instances representing the topics to assign
/// - `rooms`: A slice of `Room` instances representing the rooms to assign the topics to
/// - `existing_timeslots`: A slice of `TimeSlot` instances representing the existing timeslots
/// - `schedule_id`: The ID of the schedule to assign the timeslots to
///
/// # Returns
/// A `Result` containing a vector of `TimeSlot` instances with the topics assigned if successful,
/// otherwise a `ScheduleErr` error.
///
/// # Errors
/// If an I/O error occurs, a `ScheduleErr` error is returned.
pub async fn assign_topics_to_timeslots(
    topics: &[Topic],
    rooms: &[Room],
    existing_timeslots: &[TimeSlot],
    db_pool: &Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut topic_index = 0;
    let mut used_topics = HashSet::new();

    for slot in existing_timeslots {
        let mut assignments = Vec::new();

        // Get existing assignments for this timeslot
        let existing_assignments = sqlx::query_as::<_, TimeslotAssignment>(
            "SELECT * FROM timeslot_assignments WHERE time_slot_id = $1"
        )
            .bind(slot.id.unwrap_or_default())
            .fetch_all(db_pool)
            .await?;

        let used_rooms: HashSet<i32> = existing_assignments
            .iter()
            .map(|a| a.room_id)
            .collect();

        for assignment in &existing_assignments {
            used_topics.insert(assignment.topic_id);
        }

        // Only assign to available rooms
        for room in rooms {
            let room_id = room.id.ok_or("Room missing ID")?;
            if !used_rooms.contains(&room_id) && topic_index < topics.len() {
                while topic_index < topics.len() {
                    let topic = &topics[topic_index];
                    let topic_id = topic.id.ok_or("Topic missing ID")?;

                    if !used_topics.contains(&topic_id) {
                        assignments.push(TimeslotAssignmentForm {
                            speaker_id: topic.speaker_id,
                            topic_id,
                            room_id,
                            old_room_id: 0,
                        });
                        used_topics.insert(topic_id);
                        topic_index += 1;
                        break;
                    }
                    topic_index += 1;
                }
            }
        }

        if !assignments.is_empty() {
            insert_assignments(db_pool, slot.id.unwrap_or_default(), assignments).await?;
        }
    }

    Ok(())
}

async fn insert_assignments(
    db_pool: &Pool<Postgres>,
    timeslot_id: i32,
    assignments: Vec<TimeslotAssignmentForm>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for assignment in assignments {
        sqlx::query(
            "INSERT INTO timeslot_assignments (time_slot_id, speaker_id, topic_id, room_id) VALUES ($1, $2, $3, $4)"
        )
            .bind(timeslot_id)
            .bind(assignment.speaker_id)
            .bind(assignment.topic_id)
            .bind(assignment.room_id)
            .execute(db_pool)
            .await?;
    }
    Ok(())
}

pub async fn timeslot_assignment_update(
    db_pool: &Pool<Postgres>,
    timeslot_id: i32,
    request: TimeslotRequest,
) -> Result<Vec<i32>, Box<dyn Error>> {
    let mut assignment_ids = Vec::new();
    trace!("Updating timeslot assignments: {:?}", request);

    for timeslot in request.timeslots {
        let start_time = NaiveTime::parse_from_str(&timeslot.start_time, "%H:%M")?;
        let end_time = start_time + chrono::Duration::minutes(timeslot.duration as i64);

        // Get timeslot ID
        let new_timeslot_id: i32 = sqlx::query_scalar(
            "SELECT id FROM time_slots WHERE start_time = $1 AND end_time = $2"
        )
            .bind(start_time)
            .bind(end_time)
            .fetch_one(db_pool)
            .await?;

        trace!("Timeslot ID: {:?}", timeslot_id);

        for assignment in timeslot.assignments {
            trace!("Updating from room: {:?} to new room {:?}\n", assignment.old_room_id, assignment.room_id);
            trace!("Updating from timeslot: {:?} to new timeslot {:?}\n", timeslot_id, new_timeslot_id);
            let (assignment_id, ) =
                sqlx::query_as(
                    "UPDATE timeslot_assignments
                     SET time_slot_id = $1, speaker_id = $2, topic_id = $3, room_id = $4
                     WHERE time_slot_id = $5 AND room_id = $6
                     RETURNING id"
                )
                    .bind(new_timeslot_id)
                    .bind(assignment.speaker_id)
                    .bind(assignment.topic_id)
                    .bind(assignment.room_id)
                    .bind(timeslot_id)
                    .bind(assignment.old_room_id)
                    .fetch_one(db_pool)
                    .await?;

            assignment_ids.push(assignment_id);
        }
    }
    trace!("\n\n\n");

    Ok(assignment_ids)
}