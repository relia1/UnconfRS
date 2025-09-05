use crate::models::room_model::{rooms_get, Room};
use crate::models::schedule_model::ScheduleErr;
use crate::models::sessions_model::Session;
use crate::models::timeslot_model::{timeslot_get, ExistingTimeslot, TimeslotAssignmentForm, TimeslotAssignmentSessionAdd, TimeslotRequest};
use chrono::NaiveTime;
use scheduler::{RoomTimeAssignment, ScheduleRow, SchedulerData, SessionData};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::env::var;
use std::error::Error;
use std::time::Instant;
use tracing::info;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeslotSwapRequest {
    pub timeslot_id_1: i32,
    pub timeslot_id_2: i32,
    pub room_id_1: i32,
    pub room_id_2: i32,
}

pub async fn session_already_scheduled(db_pool: &Pool<Postgres>, session_id: i32) -> Result<bool, ScheduleErr> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM timeslot_assignments WHERE session_id = $1",
        session_id,
    )
        .fetch_one(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    Ok(count.unwrap_or(0) > 0)
}

pub async fn space_to_add_session(db_pool: &Pool<Postgres>) -> Result<bool, ScheduleErr> {
    let total_possible_timeslots = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM time_slots ts CROSS JOIN rooms r",
    )
        .fetch_one(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    let assigned_slots = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM timeslot_assignments",
    )
        .fetch_one(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    Ok(assigned_slots < total_possible_timeslots)
}

pub async fn get_all_unassigned_timeslots(db_pool: &Pool<Postgres>) -> Result<Vec<TimeslotAssignmentSessionAdd>, ScheduleErr> {
    let unassigned_timeslots = sqlx::query_as!(
        TimeslotAssignmentSessionAdd,
        r#"
        SELECT
            ts.id as time_slot_id,
            NULL::INTEGER as session_id,
            r.id as room_id
        FROM time_slots ts
        CROSS JOIN rooms r
        WHERE NOT EXISTS (
            SELECT 1
            FROM timeslot_assignments ta
            WHERE ta.time_slot_id = ts.id
            AND ta.room_id = r.id
        )
        ORDER BY ts.start_time, r.id
        "#
    )
        .fetch_all(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    Ok(unassigned_timeslots)
}

pub enum SchedulingMethod {
    Original,
    LocalSearch,
}

impl SchedulingMethod {
    
    #[allow(clippy::new_without_default)]
    pub fn new() -> SchedulingMethod {
        let scheduling_method = var("SCHEDULING_METHOD")
            .unwrap_or(String::from("Original"));

        match scheduling_method.to_lowercase().as_str() {
            "original" => SchedulingMethod::Original,
            "localsearch" => SchedulingMethod::LocalSearch,
            _ => SchedulingMethod::Original,
        }
    }
}

#[derive(Debug)]
pub struct UnassignedSession {
    pub session_id: i32,
    pub tag_id: Option<i32>,
}

pub struct SessionAssignmentData {
    pub already_assigned_room_time_associations: Vec<RoomTimeAssignment>,
    pub available_room_time_associations: Vec<TimeslotAssignmentSessionAdd>,
    pub unassigned_sessions: Vec<UnassignedSession>,
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
    _rooms: &[Room],
    _existing_timeslots: &[ExistingTimeslot],
    db_pool: &Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // alias ta for the table timeslot_assignments
    // alias uv for user_votes table
    // alias st for session_tags
    let all_assigned_sessions: Vec<RoomTimeAssignment> = sqlx::query_as!(
        RoomTimeAssignment,
        r#"SELECT
            ta.id as "id?",
            ta.time_slot_id as "time_slot_id!",
            ta.session_id as "session_id!",
            ta.room_id as "room_id!",
            true as "already_assigned!",
            COALESCE(COUNT(uv.session_id), 0)::INTEGER as "num_votes!",
            st.tag_id,
            s.user_id as speaker_id,
            ARRAY[]::INTEGER[] as "speaker_votes!"
        FROM timeslot_assignments ta
        JOIN user_votes uv ON ta.session_id = uv.session_id
        LEFT JOIN session_tags st ON st.session_id = ta.session_id
        LEFT JOIN sessions s ON s.id = ta.session_id
        GROUP BY ta.id, ta.time_slot_id, ta.session_id, ta.room_id, st.tag_id, s.user_id"#
    )
        .fetch_all(db_pool)
        .await?;

    tracing::trace!("all assigned sessions: {:?}", all_assigned_sessions);

    let used_sessions: HashSet<i32> = all_assigned_sessions
        .iter()
        .filter_map(|item| item.session_id)
        .collect();

    tracing::trace!("used_sessions: {:?}", used_sessions);

    let all_sessions: HashSet<i32> = sessions
        .iter()
        .filter_map(|s| s.id)
        .collect();

    tracing::trace!("all_sessions: {:?}", all_sessions);

    let free_sessions = all_sessions.difference(&used_sessions);
    tracing::trace!("free_sessions: {:?}", free_sessions);
    let free_roomtimes = get_all_unassigned_timeslots(db_pool).await?;
    tracing::trace!("free_roomtimes: {:?}", free_roomtimes);

    match SchedulingMethod::new() {
        SchedulingMethod::Original => {
            tracing::info!("Using original scheduling method");
            let pairings: Vec<(TimeslotAssignmentSessionAdd, i32)> = free_roomtimes
                .into_iter()
                .zip(free_sessions.copied())
                .collect();

            tracing::trace!("pairings: {:?}", pairings);

            original_scheduling(db_pool, pairings).await
        },
        SchedulingMethod::LocalSearch => {
            tracing::info!("Using localsearch scheduling method");
            let scheduling_data = SessionAssignmentData {
                already_assigned_room_time_associations: all_assigned_sessions,
                available_room_time_associations: free_roomtimes,
                unassigned_sessions: free_sessions
                    .map(|&session_id| {
                        let tag_id = sessions
                            .iter()
                            .find(|s| s.id == Some(session_id))
                            .and_then(|s| s.tag_id);
                        UnassignedSession { session_id, tag_id }
                    })
                    .collect(),
            };

            match local_search_scheduling(db_pool, scheduling_data).await {
                Ok(_) => {
                    Ok(())
                },
                Err(e) => {
                    tracing::info!("Error generating schedule {:?}", e);
                    Err(Box::new(ScheduleErr::IoError(e.to_string())))
                },
            }
        },
    }
}

pub async fn original_scheduling(db_pool: &Pool<Postgres>, pairings: Vec<(TimeslotAssignmentSessionAdd, i32)>) -> Result<(), Box<dyn Error + Send + Sync>> {
    for (rt, s) in pairings {
        let assignment = TimeslotAssignmentForm {
            session_id: s,
            room_id: rt.room_id,
            old_room_id: 0,
        };
        insert_assignment(db_pool, rt.time_slot_id, assignment).await?;
    }

    Ok(())
}


pub async fn local_search_scheduling(db_pool: &Pool<Postgres>, scheduling_data: SessionAssignmentData) -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing::trace!("unassigned_sessions: {:?}", scheduling_data.unassigned_sessions);
    // We should not be able to get here if rooms is None
    let rooms: Vec<Room> = rooms_get(db_pool).await?.unwrap();
    // We should not be able to get here if timeslots is Err
    let timeslots: Vec<ExistingTimeslot> = timeslot_get(db_pool).await.unwrap();
    let num_rooms = rooms.len();
    let num_timeslots = timeslots.len();

    tracing::info!("Getting session data");
    let session_and_votes: Vec<SessionData> = sqlx::query_as!(
        SessionData,
        "SELECT uv.session_id as \"session_id!\", \
        COALESCE(COUNT(*)::INTEGER, 0) as \"num_votes!\", \
        st.tag_id as \"tag_id?\", \
        s.user_id as \"speaker_id?\", \
        ARRAY[]::INTEGER[] as \"speaker_votes!\" \
        from user_votes uv \
        LEFT JOIN session_tags st ON st.session_id = uv.session_id \
        LEFT JOIN sessions s ON s.id = uv.session_id \
        GROUP BY uv.session_id, st.tag_id, s.user_id"
    )
        .fetch_all(db_pool)
        .await?;

    tracing::info!("Getting unassigned sessions");
    let unassigned_sessions: Vec<SessionData> = scheduling_data.unassigned_sessions
        .iter()
        .map(|&UnassignedSession { session_id, tag_id }| {
            let session_data = session_and_votes
                .iter()
                .find(|session_data| session_data.session_id.is_some() && session_data.session_id.unwrap() == session_id);

            let (num_votes, speaker_id, speaker_votes) = session_data
                .map(|session_data| (session_data.num_votes, session_data.speaker_id, session_data.speaker_votes.clone()))
                .unwrap_or((0, None, vec![]));

            SessionData {
                session_id: Some(session_id),
                num_votes,
                tag_id,
                speaker_id,
                speaker_votes,
            }
        })
        .collect();

    let mut scheduler_data: SchedulerData = SchedulerData {
        schedule_rows: vec![],
        capacity: (num_rooms * num_timeslots) as i32,
        unassigned_sessions,
    };

    for timeslot in timeslots {
        let mut schedule_row: ScheduleRow = ScheduleRow {
            schedule_items: vec![],
        };
        for room in &rooms {
            let item = RoomTimeAssignment {
                room_id: room.id.unwrap(),
                time_slot_id: timeslot.id,
                session_id: None,
                num_votes: 0,
                id: None,
                already_assigned: false,
                tag_id: None,
                speaker_id: None,
                speaker_votes: vec![],
            };

            schedule_row.schedule_items.push(item);
        }
        scheduler_data.schedule_rows.push(schedule_row);
    }

    for room_time_assgn in scheduling_data.already_assigned_room_time_associations {
        if let Some(schedule_item) = scheduler_data.schedule_rows
            .iter_mut()
            .flat_map(|row| row.schedule_items.iter_mut())
            .find(|item| item.room_id == room_time_assgn.room_id
                && item.time_slot_id == room_time_assgn.time_slot_id
            ) {
            schedule_item.session_id = room_time_assgn.session_id;
            schedule_item.id = room_time_assgn.id;
            schedule_item.already_assigned = room_time_assgn.already_assigned;

            if let Some(session_id) = room_time_assgn.session_id {
                schedule_item.num_votes = session_and_votes
                    .iter()
                    .find(|session_data| session_data.session_id.is_some() && session_data.session_id.unwrap() == session_id)
                    .map(|session_data| session_data.num_votes)
                    .unwrap_or(0);
            }
        }
    }

    tracing::info!("Starting scheduler");
    let start = Instant::now();
    let current_score = scheduler_data.improve_with_restarts(20);
    let best_scheduler_data = &scheduler_data;

    let formatted_schedule = best_scheduler_data.schedule_rows
        .iter()
        .map(|row| {
            row.schedule_items
                .iter()
                .map(|session| session.num_votes.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect::<Vec<_>>()
        .join("\n");

    tracing::info!("formatted schedule:\n {}", formatted_schedule);
    tracing::info!("current unassigned {:?}", best_scheduler_data.unassigned_sessions);

    let duration = start.elapsed();
    tracing::trace!("scheduling_data: {:?}", best_scheduler_data);
    tracing::trace!("duration: {:?}", duration);
    tracing::trace!("best score: {:?}", current_score);

    for schedule_row in &best_scheduler_data.schedule_rows {
        for schedule_item in &schedule_row.schedule_items {
            if schedule_item.already_assigned || schedule_item.session_id.is_none() {
                continue;
            } else {
                let assignment = TimeslotAssignmentForm {
                    session_id: schedule_item.session_id.unwrap(),
                    room_id: schedule_item.room_id,
                    old_room_id: 0,
                };

                insert_assignment(db_pool, schedule_item.time_slot_id, assignment).await?;
            }
        }
    }

    Ok(())
}

async fn insert_assignment(
    db_pool: &Pool<Postgres>,
    timeslot_id: i32,
    assignment: TimeslotAssignmentForm,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    sqlx::query(
        "INSERT INTO timeslot_assignments (time_slot_id, session_id, room_id) VALUES ($1, $2, $3)"
        )
        .bind(timeslot_id)
        .bind(assignment.session_id)
        .bind(assignment.room_id)
        .execute(db_pool)
        .await?;
    Ok(())
}

pub async fn timeslot_assignment_update(
    db_pool: &Pool<Postgres>,
    timeslot_id: i32,
    request: TimeslotRequest,
) -> Result<Vec<i32>, Box<dyn Error>> {
    let mut assignment_ids = Vec::new();
    info!("Updating timeslot assignments: {:?}", request);

    for timeslot in request.timeslots {
        let start_time = NaiveTime::parse_from_str(&timeslot.start_time, "%H:%M")?;
        let end_time = start_time + chrono::Duration::minutes(i64::from(timeslot.duration));

        // Get timeslot ID
        let new_timeslot_id: i32 =
            sqlx::query_scalar!(
                "SELECT id FROM time_slots WHERE start_time = $1 AND end_time = $2",
                start_time as _,
                end_time as _,
            )
                .fetch_one(db_pool)
                .await?;

        for assignment in timeslot.assignments {
            info!(
                "Updating from room: {:?} to new room {:?}\n",
                assignment.old_room_id,
                assignment.room_id
            );
            info!(
                "Updating from timeslot: {:?} to new timeslot {:?}\n",
                timeslot_id,
                new_timeslot_id
            );
            let assignment_id = sqlx::query_scalar!(
                "UPDATE timeslot_assignments
                SET time_slot_id = $1, session_id = $2, room_id = $3
                WHERE time_slot_id = $4 AND room_id = $5
                RETURNING id",
                new_timeslot_id,
                assignment.session_id,
                assignment.room_id,
                timeslot_id,
                assignment.old_room_id,
            )
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

    sqlx::query!(
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
        request.timeslot_id_1,
        request.room_id_1,
        request.timeslot_id_2,
        request.room_id_2,
    )
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
