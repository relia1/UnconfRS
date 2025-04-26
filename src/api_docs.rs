use crate::{
    controllers::{
        room_handler, schedule_handler, sessions_handler, timeslot_handler,
    },
    models::{
        room_model::Room, schedule_model::Schedule, sessions_model::Session,
        timeslot_model::TimeSlot,
    },
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Sessions
        sessions_handler::sessions,
        sessions_handler::get_session,
        sessions_handler::post_session,
        sessions_handler::delete_session,
        sessions_handler::update_session,
        sessions_handler::add_vote_for_session,
        sessions_handler::subtract_vote_for_session,
        // Rooms
        room_handler::rooms,
        room_handler::post_rooms,
        room_handler::delete_room,
        // Schedules
        schedule_handler::generate,
        schedule_handler::clear,
        // Timeslots
        timeslot_handler::update_timeslot,
    ),
    components(
        schemas(Session, Room, Schedule, TimeSlot)
    ),
    tags(
        (name = "Sessions", description = "Session management endpoints"),
        (name = "Rooms", description = "Room management endpoints"),
        (name = "Schedules", description = "Schedule management endpoints"),
        (name = "Timeslots", description = "Timeslot management endpoints")
    )
)]
pub struct ApiDoc;
