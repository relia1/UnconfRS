use crate::models::room_model::Room;
use crate::models::sessions_model::Session;
use crate::models::timeslot_model::{
    ExistingTimeslot, TimeslotAssignment, TimeslotAssignmentForm, TimeslotRequest,
};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::error::Error;
use tracing::trace;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeslotSwapRequest {
    pub timeslot_id_1: i32,
    pub timeslot_id_2: i32,
    pub room_id_1: i32,
    pub room_id_2: i32,
}

/// Assigns sessions to timeslots.
///
/// This function assigns sessions to timeslots based on the provided sessions, rooms, and existing
/// timeslots. The sessions are assigned to the timeslots in the order they are provided, starting
/// with the first session and moving to the next session for each room.
///
/// # Parameters
/// - `sessions`: A slice of `Session` instances representing the sessions to assign
/// - `rooms`: A slice of `Room` instances representing the rooms to assign the sessions to
/// - `existing_timeslots`: A slice of `TimeSlot` instances representing the existing timeslots
/// - `schedule_id`: The ID of the schedule to assign the timeslots to
///
/// # Returns
/// A `Result` containing a vector of `TimeSlot` instances with the sessions assigned if successful,
/// otherwise a `ScheduleErr` error.
///
/// # Errors
/// If an I/O error occurs, a `ScheduleErr` error is returned.
pub async fn assign_sessions_to_timeslots(
    sessions: &[Session],
    rooms: &[Room],
    existing_timeslots: &[ExistingTimeslot],
    db_pool: &Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut session_index = 0;
    let mut used_sessions = HashSet::new();

    for slot in existing_timeslots {
        let mut assignments = Vec::new();

        // Get existing assignments for this timeslot
        let existing_assignments = sqlx::query_as::<_, TimeslotAssignment>(
            "SELECT * FROM timeslot_assignments WHERE time_slot_id = $1",
        )
            .bind(slot.id)
            .fetch_all(db_pool)
            .await?;

        let used_rooms: HashSet<i32> = existing_assignments.iter().map(|a| a.room_id).collect();

        for assignment in &existing_assignments {
            used_sessions.insert(assignment.session_id);
        }

        // Only assign to available rooms
        for room in rooms {
            let room_id = room.id.ok_or("Room missing ID")?;
            if !used_rooms.contains(&room_id) && session_index < sessions.len() {
                while session_index < sessions.len() {
                    let session = &sessions[session_index];
                    let session_id = session.id.ok_or("Session missing ID")?;

                    if !used_sessions.contains(&session_id) {
                        assignments.push(TimeslotAssignmentForm {
                            session_id,
                            room_id,
                            old_room_id: 0,
                        });
                        used_sessions.insert(session_id);
                        session_index += 1;
                        break;
                    }
                    session_index += 1;
                }
            }
        }

        if !assignments.is_empty() {
            insert_assignments(db_pool, slot.id, assignments).await?;
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
            "INSERT INTO timeslot_assignments (time_slot_id, session_id, room_id) VALUES ($1, $2, $3)"
        )
            .bind(timeslot_id)
            .bind(assignment.session_id)
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
        let end_time = start_time + chrono::Duration::minutes(i64::from(timeslot.duration));

        // Get timeslot ID
        let new_timeslot_id: i32 =
            sqlx::query_scalar("SELECT id FROM time_slots WHERE start_time = $1 AND end_time = $2")
                .bind(start_time)
                .bind(end_time)
                .fetch_one(db_pool)
                .await?;

        for assignment in timeslot.assignments {
            trace!(
                "Updating from room: {:?} to new room {:?}\n",
                assignment.old_room_id,
                assignment.room_id
            );
            trace!(
                "Updating from timeslot: {:?} to new timeslot {:?}\n",
                timeslot_id,
                new_timeslot_id
            );
            let (assignment_id, ) = sqlx::query_as(
                "UPDATE timeslot_assignments
                     SET time_slot_id = $1, session_id = $2, room_id = $3
                     WHERE time_slot_id = $4 AND room_id = $5
                     RETURNING id",
            )
                .bind(new_timeslot_id)
                .bind(assignment.session_id)
                .bind(assignment.room_id)
                .bind(timeslot_id)
                .bind(assignment.old_room_id)
                .fetch_one(db_pool)
                .await?;

            assignment_ids.push(assignment_id);
        }
    }

    Ok(assignment_ids)
}

pub async fn timeslot_assignment_swap(
    db_pool: &Pool<Postgres>,
    request: TimeslotSwapRequest,
) -> Result<(), Box<dyn Error>> {
    let mut tx = db_pool.begin().await?;

    sqlx::query(
        "UPDATE timeslot_assignments t1
        SET
            session_id = t2.session_id
        FROM (
            SELECT id, session_id
            FROM timeslot_assignments
            WHERE (time_slot_id, room_id) IN (($1, $2), ($3, $4))
        ) t2
        WHERE t1.id != t2.id
        AND (t1.time_slot_id, t1.room_id) IN (($1, $2), ($3, $4))",
    )
        .bind(request.timeslot_id_1)
        .bind(request.room_id_1)
        .bind(request.timeslot_id_2)
        .bind(request.room_id_2)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
