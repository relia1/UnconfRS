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

        println!("Creating C({}, {}) combinations", num_sessions, num_slots);
        // Get all combinations of the number of spots on the schedule
        let combinations: Vec<_> = all_sessions.iter()
            .combinations(num_slots)
            .collect();

        println!("Combinations created: {}\n", combinations.len());

        println!("Processing the {} combinations in parallel ({} permutations sequentially within each thread)", combinations.len(), factorial(num_slots).to_formatted_string(&Locale::en));
        let best_data = std::sync::Mutex::new((f32::MAX, self.clone()));
        let worst_data = std::sync::Mutex::new((f32::MIN, self.clone()));

        // For each combination in parallel create serially all permutations and score them while
        // collecting statistics on best/worst scores/schedules
        let results: Vec<f32> = combinations
            .par_iter()
            .flat_map(|combination| {
                let combination_values: Vec<_> = combination.iter()
                    .map(|&&x| x)
                    .collect();

                // Process all permutations of this combination sequentially within this thread
                // Each thread will have its own local best/worst scores/schedules and will sync up
                // with the 'global' mutex's after they have performed their local work
                let mut local_best = (f32::MAX, self.clone());
                let mut local_worst = (f32::MIN, self.clone());
                let mut scores = Vec::new();

                for permutation in combination_values.clone().into_iter().permutations(num_slots) {
                    let mut test_data = self.clone();
                    test_data.unassigned_sessions.clear();

                    for (i, &session) in permutation.iter().enumerate() {
                        let (row, col) = swappable_positions[i];
                        test_data.schedule_rows[row].schedule_items[col].session_id = session.0;
                        test_data.schedule_rows[row].schedule_items[col].num_votes = session.1;
                    }

                    let used_sessions: std::collections::HashSet<_> = permutation.iter()
                        .map(|session| (session.0, session.1))
                        .collect();

                    for session in &all_sessions {
                        if !used_sessions.contains(session) {
                            test_data.unassigned_sessions.push(SessionVotes {
                                session_id: session.0,
                                num_votes: session.1,
                            });
                        }
                    }

                    let score = test_data.score();
                    scores.push(score);

                    // Update this thread's best/worst score/schedule
                    if score < local_best.0 {
                        local_best = (score, test_data.clone());
                    }

                    if score > local_worst.0 {
                        local_worst = (score, test_data.clone());
                    }
                }

                // Update the global best/worst score/schedule
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