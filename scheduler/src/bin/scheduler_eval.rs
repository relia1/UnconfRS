use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;
use scheduler::utils::*;
use scheduler::SessionVotes;

struct BruteForceResults {
    scores: Vec<f32>,
    best_schedule: scheduler::SchedulerData,
    worst_schedule: scheduler::SchedulerData,
    best_score: f32,
    worst_score: f32,
}

struct SchedulerResults {
    scores: Vec<f32>,
    best_schedule: scheduler::SchedulerData,
    worst_schedule: scheduler::SchedulerData,
    best_score: Option<f32>,
    worst_score: Option<f32>,
    iterations: usize,
}

fn compare_schedulers() {
    let data = make_test_data(3, 3);

    let scheduler_results = run_scheduler(&data, 100);
    print_scheduler_results(&scheduler_results);

    let brute_force_results = run_brute_force(&data);
    print_brute_force_results(&brute_force_results);
}


fn run_scheduler(data: &scheduler::SchedulerData, iterations: usize) -> SchedulerResults {
    let mut worst_score: Option<f32> = None;
    let mut worst_schedule = data.clone();
    let mut best_schedule = data.clone();
    let mut best_score: Option<f32> = None;
    let mut scores = Vec::new();

    for _ in 0..iterations {
        let mut schedule_data = data.clone();
        let score = schedule_data.improve();
        if worst_score.is_none() {
            worst_score = Some(score);
            best_score = Some(score);
            worst_schedule = schedule_data.clone();
            best_schedule = schedule_data.clone();
        } else {
            if score < best_score.unwrap() {
                best_score = Some(score);
                best_schedule = schedule_data.clone();
            } else if score > worst_score.unwrap() {
                worst_score = Some(score);
                worst_schedule = schedule_data.clone();
            }
        }
        scores.push(score);
    }

    SchedulerResults {
        scores,
        best_schedule,
        worst_schedule,
        best_score,
        worst_score,
        iterations,
    }
}

fn run_brute_force(data: &scheduler::SchedulerData) -> BruteForceResults {
    println!("=== BRUTE FORCE EVALUATION ===");
    data.brute_force_all_assignments()
}

trait BruteForceScheduler {
    fn brute_force_all_assignments(&self) -> BruteForceResults;
}

impl BruteForceScheduler for scheduler::SchedulerData {
    fn brute_force_all_assignments(&self) -> BruteForceResults {
        let swappable_positions: Vec<(usize, usize)> = self.get_swappable_sessions();
        let mut all_sessions = Vec::new();

        for &(row, col) in &swappable_positions {
            let slot = &self.schedule_rows[row].schedule_items[col];
            if slot.session_id.is_some() {
                all_sessions.push((slot.session_id, slot.num_votes));
            }
        }

        for session in &self.unassigned_sessions {
            all_sessions.push((session.session_id, session.num_votes));
        }

        let num_slots = swappable_positions.len();
        let num_sessions = all_sessions.len();

        println!("Sessions: {}, Slots: {}\n", num_sessions, num_slots);

        // Calculate capacity for each time slot
        let time_slot_capacities = get_time_slot_capacities(self, &swappable_positions);

        println!("Creating C({}, {}) combinations", num_sessions, num_slots);
        let combinations: Vec<_> = all_sessions.iter()
            .combinations(num_slots)
            .collect();

        println!("Combinations created: {}\n", combinations.len().to_formatted_string(&Locale::en));

        // Calculate total assignments
        let total_assignments = combinations.len() *
            num_of_ways_to_group(num_slots, &time_slot_capacities);

        println!("Processing {} combinations with {} time slot assignments each = {} total assignments",
            combinations.len().to_formatted_string(&Locale::en),
            num_of_ways_to_group(num_slots, &time_slot_capacities).to_formatted_string(&Locale::en),
            total_assignments.to_formatted_string(&Locale::en));

        let best_data = std::sync::Mutex::new((f32::MAX, self.clone()));
        let worst_data = std::sync::Mutex::new((f32::MIN, self.clone()));

        let results: Vec<f32> = combinations
            .par_iter()
            .flat_map(|combination| {
                let combination_values: Vec<_> = combination.iter()
                    .map(|&&x| x)
                    .collect();

                let mut local_best = (f32::MAX, self.clone());
                let mut local_worst = (f32::MIN, self.clone());
                let mut scores = Vec::new();

                // Generate all the ways to assign sessions to time slots
                let time_slot_assignments = generate_time_slot_assignments(
                    self,
                    &combination_values,
                    &time_slot_capacities,
                    &swappable_positions,
                );

                for assignment in time_slot_assignments {
                    let mut test_data = self.clone();
                    test_data.unassigned_sessions.clear();

                    // Apply the assignment
                    for (session, (row, col)) in assignment.iter() {
                        test_data.schedule_rows[*row].schedule_items[*col].session_id = session.0;
                        test_data.schedule_rows[*row].schedule_items[*col].num_votes = session.1;
                    }

                    // Add unused sessions to unassigned
                    let used_sessions: std::collections::HashSet<_> = assignment.iter()
                        .map(|(session, _)| *session)
                        .collect();

                    for &session in &all_sessions {
                        if !used_sessions.contains(&session) {
                            test_data.unassigned_sessions.push(SessionVotes {
                                session_id: session.0,
                                num_votes: session.1,
                            });
                        }
                    }

                    let score = test_data.score();
                    scores.push(score);

                    if score < local_best.0 {
                        local_best = (score, test_data.clone());
                    }

                    if score > local_worst.0 {
                        local_worst = (score, test_data.clone());
                    }
                }

                // Update global best/worst
                {
                    let mut best = best_data.lock().unwrap();
                    if local_best.0 < best.0 {
                        *best = local_best;
                    }
                }

                {
                    let mut worst = worst_data.lock().unwrap();
                    if local_worst.0 > worst.0 {
                        *worst = local_worst;
                    }
                }

                scores
            })
            .collect();

        let best = best_data.into_inner().unwrap();
        let worst = worst_data.into_inner().unwrap();

        BruteForceResults {
            scores: results,
            best_schedule: best.1,
            worst_schedule: worst.1,
            best_score: best.0,
            worst_score: worst.0,
        }
    }
}

// Get the number of swappable positions for each time slot
fn get_time_slot_capacities(
    data: &scheduler::SchedulerData,
    swappable_positions: &[(usize, usize)],
) -> Vec<usize> {
    let mut capacities = vec![0; data.schedule_rows.len()];
    for &(row, _) in swappable_positions {
        capacities[row] += 1;
    }
    capacities
}

// Generate all the ways to assign sessions to time slots
// Returns a vector of schedule assignments
fn generate_time_slot_assignments(
    data: &scheduler::SchedulerData,
    sessions: &[(Option<i32>, i32)],
    time_slot_capacities: &[usize],
    swappable_positions: &[(usize, usize)],
) -> Vec<Vec<((Option<i32>, i32), (usize, usize))>> {
    let mut result = Vec::new();

    // Group swappable positions by time slot
    let mut positions_by_time_slot: Vec<Vec<(usize, usize)>> = vec![Vec::new(); data.schedule_rows.len()];
    for &pos in swappable_positions {
        positions_by_time_slot[pos.0].push(pos);
    }

    // Track which sessions have been assigned
    let mut used = vec![false; sessions.len()];
    let mut current_assignment = Vec::new();

    generate_assignments_recursive(
        sessions,
        time_slot_capacities,
        &positions_by_time_slot,
        &mut current_assignment,
        &mut used,
        0,
        &mut result,
    );

    result
}

// Recursively generate assignments using backtracking
fn generate_assignments_recursive(
    sessions: &[(Option<i32>, i32)],
    capacities: &[usize],
    positions_by_time_slot: &[Vec<(usize, usize)>],
    current_assignment: &mut Vec<((Option<i32>, i32), (usize, usize))>,
    used: &mut Vec<bool>,
    time_slot_idx: usize,
    result: &mut Vec<Vec<((Option<i32>, i32), (usize, usize))>>,
) {
    // We've assigned sessions to all time slots
    if time_slot_idx >= capacities.len() {
        result.push(current_assignment.clone());
        return;
    }

    let capacity = capacities[time_slot_idx];

    // If the time slot does not have capacity move on to the next
    if capacity == 0 {
        generate_assignments_recursive(sessions, capacities, positions_by_time_slot, current_assignment, used, time_slot_idx + 1, result);
        return;
    }

    // Find sessions that have not been assigned yet
    let available_sessions: Vec<usize> = used.iter()
        .enumerate()
        .filter_map(|(i, &is_used)| {
            if !is_used {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    // Generate all combinations of sessions for this time slot based on the capacity of the timeslot
    for combination in available_sessions.into_iter().combinations(capacity) {
        // Mark sessions as used
        for &idx in &combination {
            used[idx] = true;
        }

        // Assign sessions to positions in the time slot
        for (i, &session_idx) in combination.iter().enumerate() {
            let session = sessions[session_idx];
            let position = positions_by_time_slot[time_slot_idx][i];
            current_assignment.push((session, position));
        }

        // Recurse to next time slot
        generate_assignments_recursive(sessions, capacities, positions_by_time_slot, current_assignment, used, time_slot_idx + 1, result);

        // Backtrack: remove assignments and mark sessions as unused
        for _ in 0..capacity {
            current_assignment.pop();
        }
        for &idx in &combination {
            used[idx] = false;
        }
    }
}

// Calculates the number of ways to divide n items into groups of given capacities
fn num_of_ways_to_group(n: usize, capacities: &[usize]) -> usize {
    let mut result = factorial(n);
    for &capacity in capacities {
        if capacity > 0 {
            result /= factorial(capacity);
        }
    }
    result
}

fn factorial(number: usize) -> usize {
    let mut factorial = 1;
    for i in 1..=number {
        factorial *= i;
    }

    factorial
}

fn print_brute_force_results(brute_force_results: &BruteForceResults) {
    let num_of_brute_force_scores = brute_force_results.scores.len();
    let sum = brute_force_results.scores.iter().sum::<f32>();
    let max = brute_force_results.scores.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let min = brute_force_results.scores.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let avg = sum / num_of_brute_force_scores as f32;

    println!("Number of brute force scores: {}", num_of_brute_force_scores.to_formatted_string(&Locale::en));
    println!("Average score: {:.2}", avg);
    println!("Minimum score: {:.2}", min);
    println!("Maximum score: {:.2}", max);

    println!("Best brute force schedule (score: {:.2}): \n{}", brute_force_results.best_score, brute_force_results.best_schedule);
    println!("Worst brute force schedule (score: {:.2}): \n{}", brute_force_results.worst_score, brute_force_results.worst_schedule);
}

fn print_scheduler_results(scheduler_results: &SchedulerResults) {
    let sum = scheduler_results.scores.iter().sum::<f32>();
    let avg = sum / scheduler_results.iterations as f32;

    println!("\n\n=== SCHEDULER RESULTS ({} iterations) ===", scheduler_results.iterations);
    println!("Average score: {:.2}", avg);
    println!("Minimum score: {:.2}", scheduler_results.best_score.unwrap());
    println!("Maximum score: {:.2}\n", scheduler_results.worst_score.unwrap());

    println!("Best schedule: \n{}", scheduler_results.best_schedule);
    println!("Worst schedule: \n{}", scheduler_results.worst_schedule);
}

fn main() {
    compare_schedulers();
}