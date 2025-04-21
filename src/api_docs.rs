use crate::{
    controllers::{
        room_handler, schedule_handler, speakers_handler, timeslot_handler, topics_handler,
    },
    models::{
        room_model::Room, schedule_model::Schedule, timeslot_model::TimeSlot,
        topics_model::Topic, user_info_model::UserInfo,
    },
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Topics
        topics_handler::topics,
        topics_handler::get_topic,
        topics_handler::post_topic,
        topics_handler::delete_topic,
        topics_handler::update_topic,
        topics_handler::add_vote_for_topic,
        topics_handler::subtract_vote_for_topic,
        // Rooms
        room_handler::rooms,
        room_handler::post_rooms,
        room_handler::delete_room,
        // Speakers
        speakers_handler::speakers,
        speakers_handler::get_speaker,
        speakers_handler::post_speaker,
        speakers_handler::delete_speaker,
        speakers_handler::update_speaker,
        // Schedules
        schedule_handler::schedules,
        schedule_handler::get_schedule,
        schedule_handler::post_schedule,
        schedule_handler::generate,
        schedule_handler::clear,
        // Timeslots
        timeslot_handler::update_timeslot,
    ),
    components(
        schemas(Topic, Room, Schedule, UserInfo, TimeSlot)
    ),
    tags(
        (name = "Topics", description = "Topic management endpoints"),
        (name = "Rooms", description = "Room management endpoints"),
        (name = "Speakers", description = "Speaker management endpoints"),
        (name = "Schedules", description = "Schedule management endpoints"),
        (name = "Timeslots", description = "Timeslot management endpoints")
    )
)]
pub struct ApiDoc;
