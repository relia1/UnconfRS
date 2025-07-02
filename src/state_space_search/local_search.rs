use rand::prelude::IteratorRandom;

#[derive(Debug, Clone)]
pub struct SessionVotes {
    pub session_id: Option<i32>,
    pub num_votes: i32,
}

#[derive(Debug, Clone)]
pub struct SchedulerData {
    pub schedule_rows: Vec<ScheduleRow>,
    pub capacity: i32,
    pub unassigned_sessions: Vec<SessionVotes>,
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
}

pub enum SwapAction {
    FromSchedule((usize, usize), (usize, usize)),
    FromUnassigned((usize, usize), usize),
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
                    let (i, session) = self.unassigned_sessions
                        .iter()
                        .enumerate()
                        .choose(&mut rand::rng())
                        .unwrap();

                    schedule_item.session_id = session.session_id;
                    schedule_item.num_votes = session.num_votes;

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
            let swappable_sessions: Vec<(usize, usize)> = self.schedule_rows
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
                .collect();

            let coin_flip = rng.random_bool(0.5);
            if coin_flip {

                // Try all pair swaps between swappable positions within the schedule and the unassigned
                for i in 0..swappable_sessions.len() {
                    let pos1 = swappable_sessions[i];
                    // Tries swaps with other values within the schedule
                    for session in swappable_sessions.iter().skip(i + 1) {
                        let pos2 = *session;

                        // Perform the pair swap
                        self.swap_sessions(pos1, pos2);

                        // Evaluate the new score
                        let new_score = self.score();
                        if new_score < best_score {
                            best_score = new_score;
                            best_action = Some(SwapAction::FromSchedule(pos1, pos2));
                        }

                        // Swap back the positions
                        self.swap_sessions(pos2, pos1);
                    }

                    // Tries swaps with the unassigned sessions
                    for k in 0..self.unassigned_sessions.len() {
                        let pos2 = k;

                        // Perform the swap with the unassigned sessions
                        self.swap_with_unassigned_session(pos1, pos2);

                        // Evaluate the new score
                        let new_score = self.score();
                        if new_score < best_score {
                            best_score = new_score;
                            best_action = Some(SwapAction::FromUnassigned(pos1, pos2));
                        }

                        // Swap back the positions, needs to be pos1 then pos2 since the types are different
                        self.swap_with_unassigned_session(pos1, pos2);
                    }
                }
            } else {
                let pos1 = *swappable_sessions.choose(&mut rng).unwrap();
                let pos2 = *swappable_sessions.choose(&mut rng).unwrap();
                self.swap_sessions(pos1, pos2);
            }

            // We have gone through the entire schedule and at each position checked to see if there
            // was an improving move, if there is an improving move we check if it is a swap from
            // within the schedule (SwapAction::FromSchedule) or an improving move from the
            // unassigned list of sessions (SwapAction::FromUnassigned). At the moment if no
            // improving move was found we break, this will be changed soon to make the best
            // available move even if the schedule does get a little worse.
            match best_action {
                Some(SwapAction::FromSchedule(session_on_schedule1, session_on_schedule2)) => {
                    self.swap_sessions(session_on_schedule1, session_on_schedule2);
                    current_score = best_score;
                },
                Some(SwapAction::FromUnassigned(session_on_schedule1, unassigned_session_idx)) => {
                    self.swap_with_unassigned_session(session_on_schedule1, unassigned_session_idx);
                    current_score = best_score;
                },
                None => break,
            }
        }

        current_score
    }

    pub fn score(&mut self) -> f32 {
        let conflicting_penalty = self.penalize_conflicting_popular_sessions();
        let missing_popular_penalty = self.penalize_popular_sessions_missing();
        let late_sessions_penalty = self.penalize_late_popular_sessions();

        self.weight_scores(conflicting_penalty, missing_popular_penalty, late_sessions_penalty)
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
        // Sort the vec in descending order
        // Iterate over the vec
        // With a sliding window of 2 calculate the sum of adjacent pair products
        //      e.g. [a,b,c,d] (a * b) + (b * c) + (c * d)

        // Create a clone of the unassigned sessions so we don't modify the one we are already using
        // and iterating over in the 'improve' function
        let mut sorted_unassigned = self.unassigned_sessions.clone();
        // Sort to maximize the penalty of the sum of adjacent pair products
        sorted_unassigned.sort_by(|a, b| b.num_votes.cmp(&a.num_votes));

        sorted_unassigned
            .windows(2)
            .map(|pair| pair[0].num_votes * pair[1].num_votes)
            .sum()
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

    fn weight_scores(&self, penalty_conflicting: i32, penalty_missing: i32, penalty_late: i32) -> f32 {
        let weight_conflicting = 0.3;
        let weight_missing = 0.5;
        let weight_late = 0.2;

        weight_conflicting * penalty_conflicting as f32 +
            weight_missing * penalty_missing as f32 +
            weight_late * penalty_late as f32
    }

    fn swap_sessions(
        &mut self,
        pos1@(pos1_row, pos1_col): (usize, usize),
        pos2@(pos2_row, pos2_col): (usize, usize),
    ) {
        assert!(self.is_swappable(pos1) && self.is_swappable(pos2));

        // Get copies of the current values so we can perform the swap
        // Cannot do just mem::swap on the whole item since we only want to change the session_id and num_votes fields
        // Cannot do mem::swap on just session_id and num_votes either since we'd be holding 2 mutable references
        let session1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id;
        let votes1 = self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes;

        let session2 = self.schedule_rows[pos2_row].schedule_items[pos2_col].session_id;
        let votes2 = self.schedule_rows[pos2_row].schedule_items[pos2_col].num_votes;

        self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id = session2;
        self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes = votes2;

        self.schedule_rows[pos2_row].schedule_items[pos2_col].session_id = session1;
        self.schedule_rows[pos2_row].schedule_items[pos2_col].num_votes = votes1;
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

        let session2 = self.unassigned_sessions[unassigned_idx].session_id;
        let votes2 = self.unassigned_sessions[unassigned_idx].num_votes;

        self.schedule_rows[pos1_row].schedule_items[pos1_col].session_id = session2;
        self.schedule_rows[pos1_row].schedule_items[pos1_col].num_votes = votes2;

        self.unassigned_sessions[unassigned_idx].session_id = session1;
        self.unassigned_sessions[unassigned_idx].num_votes = votes1;
    }
}
