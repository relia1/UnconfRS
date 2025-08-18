use rand::prelude::IteratorRandom;
use std::cmp::max;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: Option<i32>,
    pub num_votes: i32,
    pub tag_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct SchedulerData {
    pub schedule_rows: Vec<ScheduleRow>,
    pub capacity: i32,
    pub unassigned_sessions: Vec<SessionData>,
}

#[derive(Debug, Clone)]
pub struct ScheduleRow {
    pub schedule_items: Vec<RoomTimeAssignment>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct RoomTimeAssignment {
    pub room_id: i32,
    pub time_slot_id: i32,
    pub session_id: Option<i32>,
    pub id: Option<i32>,
    pub already_assigned: bool,
    pub num_votes: i32,
    pub tag_id: Option<i32>,
}

#[derive(Clone)]
pub enum SwapAction {
    FromSchedule((usize, usize), (usize, usize)),
    FromUnassigned((usize, usize), usize),
}

impl Display for SchedulerData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (row_idx, row) in self.schedule_rows.iter().enumerate() {
            write!(f, "Row {}: ", row_idx + 1)?;

            let votes: Vec<String> = row.schedule_items
                .iter()
                .map(|session| {
                    match session.session_id {
                        Some(_) => session.num_votes.to_string(),
                        None => "-".to_string(),
                    }
                })
                .collect();

            writeln!(f, "[{}]", votes.join(", "))?;
        }

        if !self.unassigned_sessions.is_empty() {
            write!(f, "Unassigned: ")?;
            let unassigned: Vec<String> = self.unassigned_sessions
                .iter()
                .map(|session| session.num_votes.to_string())
                .collect();
            writeln!(f, "{}", unassigned.join(", "))?;
        }

        Ok(())
    }
}

impl SchedulerData {
    pub fn randomly_fill_available_spots(&mut self) {
        // Iterate through each time slot row in the schedule
        // For each row check each room assignment
        // Skip any room assignments that already have sessions assigned (already_assigned being true)
        // For empty slots randomly choose sessions from the unassigned sessions list
        // Assign the chosen session's session_id and num_votes to the room assignment
        // Remove the chosen session from the unassigned list
        for schedule_row in &mut self.schedule_rows {
            for schedule_item in &mut schedule_row.schedule_items {
                if schedule_item.already_assigned {
                    continue;
                } else {
                    // If there are not anymore unassigned sessions we are done
                    if self.unassigned_sessions.is_empty() {
                        return;
                    }
                    let (i, session) = self.unassigned_sessions
                        .iter()
                        .enumerate()
                        .choose(&mut rand::rng())
                        .unwrap();

                    schedule_item.session_id = session.session_id;
                    schedule_item.num_votes = session.num_votes;
                    schedule_item.tag_id = session.tag_id;

                    self.unassigned_sessions.swap_remove(i);
                }
            }
        }
    }

    pub fn improve(&mut self) -> f32 {
        use rand::{seq::IndexedRandom, Rng};
        let mut rng = rand::rng();

        // Start with randomly assigned schedule (preserves already assigned)
        self.randomly_fill_available_spots();

        let mut current_score = self.score();
        let max_iterations = 3 * self.capacity * self.capacity;

        let mut best_score = current_score;
        let mut best_action: Option<SwapAction> = None;
        for _ in 0..max_iterations {
            // Get only the swappable positions
            let swappable_sessions: Vec<(usize, usize)> = self.get_swappable_sessions();

            let coin_flip = rng.random_bool(1.0);
            if coin_flip {

                // Try all pair swaps between swappable positions within the schedule and the unassigned
                for i in 0..swappable_sessions.len() {
                    let pos1 = swappable_sessions[i];
                    // Tries swaps with other values within the schedule
                    for session in swappable_sessions.iter().skip(i + 1) {
                        let pos2 = *session;

                        // Perform the pair swap
                        let action = SwapAction::FromSchedule(pos1, pos2);
                        self.apply_action(&action);

                        // Evaluate the new score
                        let new_score = self.score();
                        if new_score < best_score {
                            best_score = new_score;
                            best_action = Some(action.clone());
                        }

                        // Swap back the positions
                        self.reverse_action(&action);
                    }

                    // Tries swaps with the unassigned sessions
                    for k in 0..self.unassigned_sessions.len() {
                        let pos2 = k;

                        // Perform the swap with the unassigned sessions
                        let action = SwapAction::FromUnassigned(pos1, pos2);
                        self.apply_action(&action);

                        // Evaluate the new score
                        let new_score = self.score();
                        if new_score < best_score {
                            best_score = new_score;
                            best_action = Some(action.clone());
                        }

                        // Swap back the positions, needs to be pos1 then pos2 since the types are different
                        self.reverse_action(&action);
                    }
                }
            } else {
                let pos1 = *swappable_sessions.choose(&mut rng).unwrap();
                let unassgned_sessions_len = self.unassigned_sessions.len();
                let swappable_sessions_len = swappable_sessions.len();
                let total_sessions_len = unassgned_sessions_len + swappable_sessions_len;
                let chance_random_session_is_unassigned = unassgned_sessions_len as f64 / total_sessions_len as f64;

                if !self.unassigned_sessions.is_empty() && rng.random_bool(chance_random_session_is_unassigned) {
                    // Swap with unassigned session
                    let unassigned_idx = rng.random_range(0..self.unassigned_sessions.len());
                    let action = SwapAction::FromUnassigned(pos1, unassigned_idx);
                    self.apply_action(&action);
                } else {
                    // Swap with another session in the schedule
                    let pos2 = *swappable_sessions.choose(&mut rng).unwrap();
                    let action = SwapAction::FromSchedule(pos1, pos2);
                    self.apply_action(&action);
                }
            }

            // We have gone through the entire schedule and at each position checked to see if there
            // was an improving move, if there is an improving move we check if it is a swap from
            // within the schedule (SwapAction::FromSchedule) or an improving move from the
            // unassigned list of sessions (SwapAction::FromUnassigned). At the moment if no
            // improving move was found we break, this will be changed soon to make the best
            // available move even if the schedule does get a little worse.
            match best_action.as_ref() {
                Some(action) => {
                    self.apply_action(action);
                    current_score = best_score;
                },
                None => {
                    continue;
                },
            }

            assert!(best_score >= current_score);
        }

        current_score
    }

    pub fn score(&mut self) -> f32 {
        let conflicting_penalty = self.penalize_conflicting_popular_sessions();
        let missing_popular_penalty = self.penalize_popular_sessions_missing();
        let late_sessions_penalty = self.penalize_late_popular_sessions();
        let same_tag_penalty = self.penalize_same_topic_time_slots();

        self.weight_scores(conflicting_penalty, missing_popular_penalty, late_sessions_penalty, same_tag_penalty)
    }

    fn penalize_conflicting_popular_sessions(&self) -> i32 {
        // Iterate through the rows of timeslots
        // For each timeslot row calculate their penalty
        // Within each row only keep values that have session_ids and num_votes greater than 0
        // Sort the row in descending order
        // With a sliding window of 2 calculate the sum of adjacent pair products
        //      e.g. [a,b,c,d] (a * b) + (b * c) + (c * d)
        // Then sum up all the row sums to get our total penalty for all rows
        self.schedule_rows
            .iter()
            .map(|timeslot| {
                let mut assigned_sessions: Vec<&RoomTimeAssignment> = timeslot.schedule_items
                    .iter()
                    .filter(|session_assignment| session_assignment.session_id.is_some() && session_assignment.num_votes > 0)
                    .collect();

                assigned_sessions.sort_by(|a, b| b.num_votes.cmp(&a.num_votes));
                assigned_sessions
                    .windows(2)
                    .map(|pair| pair[0].num_votes * pair[1].num_votes)
                    .sum::<i32>()
            })
            .sum()
    }

    fn penalize_popular_sessions_missing(&self) -> i32 {
        let scheduled_votes: Vec<i32> = self.schedule_rows
            .iter()
            .flat_map(|row| &row.schedule_items)
            .filter(|session| session.session_id.is_some())
            .map(|session| session.num_votes)
            .collect();

        let unassigned_votes: Vec<i32> = self.unassigned_sessions
            .iter()
            .map(|session| session.num_votes)
            .collect();

        let mut penalty = 0;

        for &scheduled_vote in &scheduled_votes {
            for &unassigned_vote in &unassigned_votes {
                if unassigned_vote > scheduled_vote {
                    penalty += (unassigned_vote - scheduled_vote) * 15;
                }
            }
        }

        penalty
    }

    fn penalize_late_popular_sessions(&self) -> i32 {
        // Iterate through the rows of timeslots
        // For each timeslot row calculate their penalty
        // Within each row only keep values that have session_ids and num_votes greater than 0
        // Sort the row in descending order
        // With a sliding window of 2 calculate the sum of adjacent pair products
        //      e.g. [a,b,c,d] (a * b) + (b * c) + (c * d)
        // Then multiply the row sum by the row index to apply more of a penalty the later it is
        // Then sum up all the row sums to get our total penalty for all rows
        self.schedule_rows
            .iter()
            .enumerate()
            .map(|(row_idx, timeslot)| {
                let mut assigned_sessions: Vec<&RoomTimeAssignment> = timeslot.schedule_items
                    .iter()
                    .filter(|session_assignment| session_assignment.session_id.is_some() && session_assignment.num_votes > 0)
                    .collect();

                assigned_sessions.sort_by(|a, b| b.num_votes.cmp(&a.num_votes));
                let assigned_sessions_sum: i32 = assigned_sessions
                    .windows(2)
                    .map(|pair| pair[0].num_votes * pair[1].num_votes)
                    .sum();

                assigned_sessions_sum * (row_idx as i32)
            })
            .sum()
    }

    fn penalize_same_topic_time_slots(&self) -> i32 {
        // Iterate through the rows of timeslots
        // For each timeslot row calculate their penalty
        // Within each row only keep values that have session_ids and tag_ids
        // For each session in the row check if any after it have the same tag_id
        // If they have the same tag_id add to the penalty the product of the two
        // Sum the penalties for each of the rows
        self.schedule_rows
            .iter()
            .map(|timeslot| {
                let assigned_sessions: Vec<&RoomTimeAssignment> = timeslot.schedule_items
                    .iter()
                    .filter(|session_assignment| session_assignment.session_id.is_some() && session_assignment.tag_id.is_some())
                    .collect();

                let mut penalty = 0;
                for i in 0..assigned_sessions.len() {
                    for j in (i + 1)..assigned_sessions.len() {
                        if assigned_sessions[i].tag_id == assigned_sessions[j].tag_id {
                            // Apply penalty based on the product of votes for sessions with same tag
                            // Make sure if a session has 0 votes that a penalty is still applied
                            penalty += max(assigned_sessions[i].num_votes, 1) * max(assigned_sessions[j].num_votes, 1);
                        }
                    }
                }
                penalty
            })
            .sum()
    }

    fn weight_scores(&self, penalty_conflicting: i32, penalty_missing: i32, penalty_late: i32, penalty_same_tag: i32) -> f32 {
        let weight_conflicting = 0.5;
        let weight_missing = 0.75;
        let weight_late = 0.1;
        let weight_same_tag = 0.3;

        weight_conflicting * penalty_conflicting as f32 +
            weight_missing * penalty_missing as f32 +
            weight_late * penalty_late as f32 +
            weight_same_tag * penalty_same_tag as f32
    }

    fn apply_action(&mut self, action: &SwapAction) {
        match action {
            SwapAction::FromSchedule(session_on_schedule1, session_on_schedule2) => {
                self.swap_sessions(*session_on_schedule1, *session_on_schedule2);
            }
            SwapAction::FromUnassigned(session_on_schedule1, unassigned_session_idx) => {
                self.swap_with_unassigned_session(*session_on_schedule1, *unassigned_session_idx);
            }
        }
    }

    fn reverse_action(&mut self, action: &SwapAction) {
        self.apply_action(action);
    }

    pub fn get_swappable_sessions(&self) -> Vec<(usize, usize)> {
        self.schedule_rows
            .iter()
            .enumerate()
            .flat_map(|(row_idx, row)| {
                row.schedule_items
                    .iter()
                    .enumerate()
                    .filter_map(move |(item_idx, slot)| {
                        if !slot.already_assigned {
                            Some((row_idx, item_idx))
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }

    fn swap_sessions(
        &mut self,
        pos1@(pos1_row, pos1_col): (usize, usize),
        pos2@(pos2_row, pos2_col): (usize, usize),
    ) {
        assert!(self.is_swappable(pos1) && self.is_swappable(pos2));

        // Get copies of the current values so we can perform the swap
        // Cannot do just mem::swap on the whole item since we only want to change the session_id, num_votes, and tag_id fields
        // Cannot do mem::swap on just session_id, num_votes, and tag_id either since we'd be holding 2 mutable references
        let session1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id;
        let votes1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes;
        let tag1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].tag_id;

        let session2 = self.schedule_rows[pos2_row].schedule_items[pos2_col].session_id;
        let votes2 = self.schedule_rows[pos2_row].schedule_items[pos2_col].num_votes;
        let tag2 = self.schedule_rows[pos2_row].schedule_items[pos2_col].tag_id;

        self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id = session2;
        self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes = votes2;
        self.schedule_rows[pos1_row].schedule_items[pos1_col].tag_id = tag2;

        self.schedule_rows[pos2_row].schedule_items[pos2_col].session_id = session1;
        self.schedule_rows[pos2_row].schedule_items[pos2_col].num_votes = votes1;
        self.schedule_rows[pos2_row].schedule_items[pos2_col].tag_id = tag1;
    }

    fn is_swappable(&self, pos1: (usize, usize)) -> bool {
        let (row_idx, col_idx) = pos1;
        !self.schedule_rows[row_idx].schedule_items[col_idx].already_assigned
    }

    fn swap_with_unassigned_session(
        &mut self,
        pos1 @ (pos1_row, pos1_col): (usize, usize),
        unassigned_idx: usize,
    ) {
        // Only need to check if pos1 is swappable since any unassigned item can be swapped onto the schedule
        assert!(self.is_swappable(pos1));

        // Get copies of the current values so we can perform the swap
        let session1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id;
        let votes1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes;
        let tag1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].tag_id;

        let session2 = self.unassigned_sessions[unassigned_idx].session_id;
        let votes2 = self.unassigned_sessions[unassigned_idx].num_votes;
        let tag2 = self.unassigned_sessions[unassigned_idx].tag_id;

        self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id = session2;
        self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes = votes2;
        self.schedule_rows[pos1_row].schedule_items[pos1_col].tag_id = tag2;

        self.unassigned_sessions[unassigned_idx].session_id = session1;
        self.unassigned_sessions[unassigned_idx].num_votes = votes1;
        self.unassigned_sessions[unassigned_idx].tag_id = tag1;
        self.unassigned_sessions[unassigned_idx].speaker_id = speaker1;
        self.unassigned_sessions[unassigned_idx].speaker_votes = speaker_votes1;
    }

    /// Runs the scheduler with multiple restarts to find the best solution
    ///
    /// # Parameters
    /// - `restarts`: Number of times to restart the improvement process
    ///
    /// # Returns
    /// The best score found across all restarts
    pub fn improve_with_restarts(&mut self, restarts: usize) -> f32 {
        let unmodified_data = self.clone();
        let mut best_score = self.improve();
        let mut best_data = self.clone();

        for _ in 0..restarts {
            *self = unmodified_data.clone();
            let new_score = self.improve();
            if new_score < best_score {
                best_score = new_score;
                best_data = self.clone();
            }
        }

        *self = best_data;
        best_score
    }
}

pub mod utils {
    use super::*;

    pub fn make_test_data(num_of_rooms: i32, num_of_time_slots: i32) -> SchedulerData {
        let mut schedule_rows = Vec::new();

        for time_slot in 1..=num_of_time_slots {
            let mut schedule_items = Vec::new();
            for room in 1..=num_of_rooms {
                schedule_items.push(RoomTimeAssignment {
                    room_id: room,
                    time_slot_id: time_slot,
                    session_id: None,
                    id: None,
                    already_assigned: false,
                    num_votes: 0,
                    tag_id: Some(room),
                });
            }
            schedule_rows.push(ScheduleRow { schedule_items });
        }

        // Let there be 1/3 more sessions than spots on the schedule
        let num_of_sessions: i32 = (((num_of_rooms * num_of_time_slots) as f32 * (4.0 / 3.0)) as i32) + 1;

        let mut unassigned_sessions = Vec::new();
        for i in 0..num_of_sessions {
            unassigned_sessions.push(SessionData {
                session_id: Some(i),
                num_votes: 3 * (i / num_of_rooms),
                tag_id: Some((i % 6) + 1),
            });
        }

        SchedulerData {
            schedule_rows,
            capacity: num_of_rooms * num_of_time_slots,
            unassigned_sessions,
        }
    }

    pub fn make_test_data_with_preassigned(num_of_rooms: i32, num_of_time_slots: i32) -> SchedulerData {
        let mut data = make_test_data(num_of_rooms, num_of_time_slots);

        // Mark first session in the first time slot as already assigned
        if let Some(first_schedule_row) = data.schedule_rows.first_mut() && let Some(session) = first_schedule_row.schedule_items.first_mut() {
            session.already_assigned = true;
            session.session_id = Some(999);
        }

        data
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;


    mod unit_tests {
        use super::{utils::*, *};
        use approx::assert_relative_eq;
        use std::collections::HashSet;


        #[test]
        fn test_randomly_fill_available_spots() {
            // Creates an empty schedule with 4/3 * (num_rooms * num_time_slots) unassigned sessions
            let mut data = make_test_data(3, 5);
            let number_of_sessions = data.unassigned_sessions.len() as i32;
            // Using the unassigned sessions fill in the schedule
            data.randomly_fill_available_spots();

            // Since we have an excess of sessions compared to available space on the schedule, the
            // schedule should be entirely full
            for row in &data.schedule_rows {
                for item in &row.schedule_items {
                    assert!(item.session_id.is_some());
                }
            }

            // Make sure the number of unassigned sessions is the correct number
            let expected_unassigned = number_of_sessions - data.capacity;

            assert_eq!(data.unassigned_sessions.len() as i32, expected_unassigned);
        }

        #[test]
        fn test_no_duplicate_assignments() {
            let mut data = make_test_data(3, 5);
            data.randomly_fill_available_spots();
            let mut sessions_on_schedule = HashSet::new();
            for row in &data.schedule_rows {
                for item in &row.schedule_items {
                    if let Some(id) = item.session_id {
                        assert!(sessions_on_schedule.insert(id), "Duplicate assignment {}", id);
                    }
                }
            }
        }

        #[test]
        fn test_fewer_sessions_than_spots() {
            let mut data = make_test_data(3, 5);
            // We need to remove some sessions from the unassigned list in order to not get a full schedule
            data.unassigned_sessions.truncate(13);

            data.randomly_fill_available_spots();

            // Should have 13 sessions assigned and 2 empty spots
            let assigned_count = data.schedule_rows
                .iter()
                .flat_map(|schedule_row| &schedule_row.schedule_items)
                .filter(|session| session.session_id.is_some())
                .count();

            assert_eq!(assigned_count, 13);
            assert_eq!(data.unassigned_sessions.len(), 0);
        }

        #[test]
        fn test_swap_sessions() {
            let mut data = make_test_data(3, 5);
            data.randomly_fill_available_spots();

            let pos1 = (0, 0);
            let pos2 = (1, 1);

            let session1_before = data.schedule_rows[pos1.0].schedule_items[pos1.1].session_id;
            let votes1_before = data.schedule_rows[pos1.0].schedule_items[pos1.1].num_votes;
            let session2_before = data.schedule_rows[pos2.0].schedule_items[pos2.1].session_id;
            let votes2_before = data.schedule_rows[pos2.0].schedule_items[pos2.1].num_votes;

            data.swap_sessions(pos1, pos2);

            assert_eq!(data.schedule_rows[pos1.0].schedule_items[pos1.1].session_id, session2_before);
            assert_eq!(data.schedule_rows[pos1.0].schedule_items[pos1.1].num_votes, votes2_before);
            assert_eq!(data.schedule_rows[pos2.0].schedule_items[pos2.1].session_id, session1_before);
            assert_eq!(data.schedule_rows[pos2.0].schedule_items[pos2.1].num_votes, votes1_before);
        }

        #[test]
        fn test_swap_with_unassigned_session() {
            let mut data = make_test_data(3, 5);
            data.randomly_fill_available_spots();

            let pos1 = (0, 0);
            let unassigned_idx = 0;

            let session1_before = data.schedule_rows[pos1.0].schedule_items[pos1.1].session_id;
            let votes1_before = data.schedule_rows[pos1.0].schedule_items[pos1.1].num_votes;
            let session2_before = data.unassigned_sessions[unassigned_idx].session_id;
            let votes2_before = data.unassigned_sessions[unassigned_idx].num_votes;

            data.swap_with_unassigned_session(pos1, unassigned_idx);

            assert_eq!(data.schedule_rows[pos1.0].schedule_items[pos1.1].session_id, session2_before);
            assert_eq!(data.schedule_rows[pos1.0].schedule_items[pos1.1].num_votes, votes2_before);
            assert_eq!(data.unassigned_sessions[unassigned_idx].session_id, session1_before);
            assert_eq!(data.unassigned_sessions[unassigned_idx].num_votes, votes1_before);
        }

        #[test]
        fn test_is_swappable() {
            let data = make_test_data_with_preassigned(3, 5);

            // Already assigned position should not be swappable
            assert!(!data.is_swappable((0, 0)));

            // Positions not already assigned should be swappable
            assert!(data.is_swappable((0, 1)));
            assert!(data.is_swappable((1, 0)));
        }

        #[test]
        fn test_penalize_conflicting_popular_sessions() {
            let mut data = make_test_data(3, 3);
            data.randomly_fill_available_spots();

            // Time slot1
            data.schedule_rows[0].schedule_items[0].num_votes = 10;
            data.schedule_rows[0].schedule_items[1].num_votes = 8;
            data.schedule_rows[0].schedule_items[2].num_votes = 5;

            // Time slot 2
            data.schedule_rows[1].schedule_items[0].num_votes = 3;
            data.schedule_rows[1].schedule_items[1].num_votes = 7;
            data.schedule_rows[1].schedule_items[2].num_votes = 5;

            // Time slot 3
            data.schedule_rows[2].schedule_items[0].num_votes = 4;
            data.schedule_rows[2].schedule_items[1].num_votes = 0;
            data.schedule_rows[2].schedule_items[2].num_votes = 7;

            let penalty = data.penalize_conflicting_popular_sessions();
            assert_eq!(penalty, 198);
        }

        #[test]
        fn test_penalize_popular_sessions_missing() {
            let mut data = make_test_data(3, 3);
            data.randomly_fill_available_spots();
            data.unassigned_sessions = vec![
                SessionData { session_id: Some(1), num_votes: 10, tag_id: Some(1) },
                SessionData { session_id: Some(2), num_votes: 8, tag_id: Some(2) },
                SessionData { session_id: Some(3), num_votes: 12, tag_id: Some(3) },
                SessionData { session_id: Some(3), num_votes: 7, tag_id: Some(4) },
            ];

            // Time slot1
            data.schedule_rows[0].schedule_items[0].num_votes = 10;
            data.schedule_rows[0].schedule_items[1].num_votes = 8;
            data.schedule_rows[0].schedule_items[2].num_votes = 5;

            // Time slot 2
            data.schedule_rows[1].schedule_items[0].num_votes = 3;
            data.schedule_rows[1].schedule_items[1].num_votes = 7;
            data.schedule_rows[1].schedule_items[2].num_votes = 5;

            // Time slot 3
            data.schedule_rows[2].schedule_items[0].num_votes = 4;
            data.schedule_rows[2].schedule_items[1].num_votes = 0;
            data.schedule_rows[2].schedule_items[2].num_votes = 7;

            let penalty = data.penalize_popular_sessions_missing();

            assert_eq!(penalty, 2145);
        }

        #[test]
        fn test_penalize_late_popular_sessions() {
            let mut data = make_test_data(3, 3);
            data.randomly_fill_available_spots();

            // Time slot1
            data.schedule_rows[0].schedule_items[0].num_votes = 10;
            data.schedule_rows[0].schedule_items[1].num_votes = 8;
            data.schedule_rows[0].schedule_items[2].num_votes = 5;

            // Time slot 2
            data.schedule_rows[1].schedule_items[0].num_votes = 3;
            data.schedule_rows[1].schedule_items[1].num_votes = 7;
            data.schedule_rows[1].schedule_items[2].num_votes = 5;

            // Time slot 3
            data.schedule_rows[2].schedule_items[0].num_votes = 4;
            data.schedule_rows[2].schedule_items[1].num_votes = 0;
            data.schedule_rows[2].schedule_items[2].num_votes = 7;

            let penalty = data.penalize_late_popular_sessions();

            assert_eq!(penalty, 106);
        }

        #[test]
        fn test_weight_scores() {
            let data = make_test_data(2, 2);
            let result = data.weight_scores(198, 256, 106, 0);

            // Expect: 0.3 * 198 + 0.5 * 256 + 0.2 * 106 = 59.4 + 128 + 21.2 = 208.6
            assert_relative_eq!(result, 301.6);
        }

        #[test]
        fn test_score_calculation() {
            let mut data = make_test_data(3, 3);
            data.randomly_fill_available_spots();
            data.unassigned_sessions = vec![
                SessionData { session_id: Some(1), num_votes: 10, tag_id: Some(1) },
                SessionData { session_id: Some(2), num_votes: 8, tag_id: Some(2) },
                SessionData { session_id: Some(3), num_votes: 12, tag_id: Some(3) },
                SessionData { session_id: Some(3), num_votes: 7, tag_id: Some(4) },
            ];

            // Time slot1
            data.schedule_rows[0].schedule_items[0].num_votes = 10;
            data.schedule_rows[0].schedule_items[1].num_votes = 8;
            data.schedule_rows[0].schedule_items[2].num_votes = 5;

            // Time slot 2
            data.schedule_rows[1].schedule_items[0].num_votes = 3;
            data.schedule_rows[1].schedule_items[1].num_votes = 7;
            data.schedule_rows[1].schedule_items[2].num_votes = 5;

            // Time slot 3
            data.schedule_rows[2].schedule_items[0].num_votes = 4;
            data.schedule_rows[2].schedule_items[1].num_votes = 0;
            data.schedule_rows[2].schedule_items[2].num_votes = 7;

            let score = data.score();

            assert_relative_eq!(score, 1718.35);
        }

        #[test]
        fn test_improve_reduces_score() {
            let mut data = make_test_data(3, 5);
            data.randomly_fill_available_spots();

            let initial_score = data.score();
            let final_score = data.improve();

            // Score should be reduced or at least not worse
            assert!(final_score <= initial_score);
        }

        #[test]
        fn test_improve_preserves_already_assigned() {
            let mut data = make_test_data_with_preassigned(3, 5);
            let original_session_id = data.schedule_rows[0].schedule_items[0].session_id;
            let original_num_votes = data.schedule_rows[0].schedule_items[0].num_votes;

            data.improve();

            // The already assigned session remains unchanged
            assert_eq!(data.schedule_rows[0].schedule_items[0].session_id, original_session_id);
            assert_eq!(data.schedule_rows[0].schedule_items[0].num_votes, original_num_votes);
            assert!(data.schedule_rows[0].schedule_items[0].already_assigned);
        }

        #[test]
        fn test_empty_schedule() {
            let mut data = SchedulerData {
                schedule_rows: vec![],
                capacity: 0,
                unassigned_sessions: vec![],
            };

            data.randomly_fill_available_spots();
            let score = data.score();

            assert_relative_eq!(score, 0.0);
        }

        #[test]
        fn test_single_room_single_time_slot() {
            let mut data = make_test_data(1, 1);
            data.randomly_fill_available_spots();

            assert!(data.schedule_rows[0].schedule_items[0].session_id.is_some());
            assert!(data.unassigned_sessions.len() > 0);
        }
    }

    #[cfg(test)]
    mod scheduler_quality_tests {
        use super::{utils::*, *};
        use approx::assert_relative_eq;

        #[test]
        fn test_improvement_over_random() {
            let mut data = make_test_data(3, 5);
            data.randomly_fill_available_spots();

            let initial_score = data.score();
            let final_score = data.improve();

            assert!(final_score <= initial_score);
        }

        #[test]
        fn test_optimal_scenario() {
            let mut data = SchedulerData {
                schedule_rows: vec![
                    ScheduleRow {
                        schedule_items: vec![
                            RoomTimeAssignment { room_id: 1, time_slot_id: 1, session_id: None, id: None, already_assigned: false, num_votes: 0, tag_id: None },
                            RoomTimeAssignment { room_id: 2, time_slot_id: 1, session_id: None, id: None, already_assigned: false, num_votes: 0, tag_id: None },
                            RoomTimeAssignment { room_id: 3, time_slot_id: 1, session_id: None, id: None, already_assigned: false, num_votes: 0, tag_id: None },
                        ]
                    },
                    ScheduleRow {
                        schedule_items: vec![
                            RoomTimeAssignment { room_id: 1, time_slot_id: 2, session_id: None, id: None, already_assigned: false, num_votes: 0, tag_id: None },
                            RoomTimeAssignment { room_id: 2, time_slot_id: 2, session_id: None, id: None, already_assigned: false, num_votes: 0, tag_id: None },
                            RoomTimeAssignment { room_id: 3, time_slot_id: 2, session_id: None, id: None, already_assigned: false, num_votes: 0, tag_id: None },
                        ]
                    },
                ],
                capacity: 6,
                unassigned_sessions: vec![
                    SessionData { session_id: Some(1), num_votes: 12, tag_id: Some(1) },
                    SessionData { session_id: Some(2), num_votes: 10, tag_id: Some(2) },
                    SessionData { session_id: Some(3), num_votes: 8, tag_id: Some(3) },
                    SessionData { session_id: Some(4), num_votes: 6, tag_id: Some(4) },
                    SessionData { session_id: Some(5), num_votes: 4, tag_id: Some(5) },
                    SessionData { session_id: Some(6), num_votes: 2, tag_id: Some(6) },
                ],
            };

            let final_score = data.improve();

            // All sessions should be scheduled
            assert_eq!(data.unassigned_sessions.len(), 0);
            assert_relative_eq!(final_score, 97.6);
        }
    }
}