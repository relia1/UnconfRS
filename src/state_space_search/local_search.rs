use rand::prelude::IteratorRandom;

#[derive(Debug, Clone)]
pub struct SessionVotes {
    pub session_id: i32,
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

impl SchedulerData {
    pub fn randomly_fill_available_spots(&mut self) {
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

                    schedule_item.session_id = Some(session.session_id);
                    schedule_item.num_votes = session.num_votes;

                    self.unassigned_sessions.swap_remove(i);
                }
            }
        }
    }

    pub fn improve(&mut self) -> f32 {
        // Start with randomly assigned schedule (preserves already assigned)
        self.randomly_fill_available_spots();

        let mut current_score = self.score();
        let max_iterations = 3 * self.capacity * self.capacity;

        for iteration in 0..max_iterations {
            let mut best_score = current_score;
            let mut best_swap: Option<((usize, usize), (usize, usize))> = None;

            // Get only the swappable positions
            let swappable_positions: Vec<(usize, usize)> = self.schedule_rows
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

            // Try all pair swaps between swappable positions
            for i in 0..swappable_positions.len() {
                for j in (i + 1)..swappable_positions.len() {
                    let pos1 = swappable_positions[i];
                    let pos2 = swappable_positions[j];

                    // Perform the pair swap
                    self.swap_sessions(pos1, pos2);

                    // Evaluate the new score
                    let new_score = self.score();
                    if new_score < best_score {
                        best_score = new_score;
                        best_swap = Some((pos1, pos2));
                    }

                    // Swap back the positions
                    self.swap_sessions(pos2, pos1);
                }
            }

            tracing::trace!("current_score: {:?}", current_score);

            // Check for improvement
            if best_score >= current_score {
                // If no improvement found we are at a local minimum
                tracing::trace!("iterations: {}", iteration);
                break;
            }

            if let Some((pos1, pos2)) = best_swap {
                self.swap_sessions(pos1, pos2);
                current_score = best_score;
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

    fn penalize_popular_sessions_missing(&mut self) -> i32 {
        self.unassigned_sessions.sort_by(|a, b| b.num_votes.cmp(&a.num_votes));

        self.unassigned_sessions
            .windows(2)
            .map(|pair| pair[0].num_votes * pair[1].num_votes)
            .sum::<i32>()
    }

    fn penalize_late_popular_sessions(&self) -> i32 {
        // Iterate through the rows of timeslots
        // For each timeslot row calculate their penalty
        // Within each row only keep values that have session_ids and num_votes greater than 0
        // Sort the row in descending order
        // With a sliding window of 2 calculate the sum of adjacent pair products
        //      e.g. [a,b,c,d] (a * b) + (b * c) + (c * d)
        // Then sum up all the row sums to get our total penalty for all rows
        self.schedule_rows
            .iter()
            .enumerate()
            .map(|(row_idx, timeslot)| {
                let assigned_sessions_sum: i32 = timeslot.schedule_items
                    .iter()
                    .filter(|session_assignment| session_assignment.session_id.is_some() && session_assignment.num_votes > 0)
                    .map(|session_assignment| session_assignment.num_votes)
                    .sum();

                assigned_sessions_sum * ((self.schedule_rows.len() - 1 - row_idx) as i32)
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

    fn swap_sessions(&mut self, pos1: (usize, usize), pos2: (usize, usize)) {
        assert!(self.is_swappable(pos1) && self.is_swappable(pos2));

        let (pos1_row, pos1_col) = pos1;
        let (pos2_row, pos2_col) = pos2;

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
}
