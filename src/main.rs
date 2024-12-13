#![feature(iterator_try_collect)]
use airplane_landing_scheduler::metaheuristics::{simulated_annealing, Problem};
use airplane_landing_scheduler::parser::parse_problem_data;
use airplane_landing_scheduler::problem::*;
use std::time::Duration;
use std::{collections::HashSet, env::args, time};

fn display_solution(problem: &LandingProblem, solution: &Solution) {
  println!(
    "TotalCost={}\tLandingCost={}\tConflictCost={}\tValid={}",
    problem.cost(solution),
    problem.landing_cost(solution),
    problem.conflict_cost(solution),
    problem.is_valid(solution)
  );
  let conflicts = problem
    .conflicts(solution)
    .flat_map(|(a, b, _)| [a, b])
    .collect::<HashSet<_>>();

  println!("-------------------------------------------------");
  println!("| ID\t| Time\t| Conf.\t| Land.\t| Separation\t|");
  println!("-------------------------------------------------");
  for (i, arrival) in solution.iter().enumerate() {
    let sep = if i == 0 {
      (
        None,
        Some(problem.separation_time_between(solution[i].plane_id, solution[i + 1].plane_id)),
      )
    } else if i == solution.len() - 1 {
      (
        Some(problem.separation_time_between(solution[i - 1].plane_id, solution[i].plane_id)),
        None,
      )
    } else {
      (
        Some(problem.separation_time_between(solution[i - 1].plane_id, solution[i].plane_id)),
        Some(problem.separation_time_between(solution[i].plane_id, solution[i + 1].plane_id)),
      )
    };
    let sep = match sep {
      (None, None) => unreachable!(),
      (None, Some(y)) => format!("- , {}", y),
      (Some(x), None) => format!("{} , -", x),
      (Some(x), Some(y)) => format!("{} , {}", x, y),
    };
    println!(
      "| {}\t| {:<6}| {}\t| {:<4}\t| {:<7}\t|",
      problem.planes[arrival.plane_id].id,
      arrival.landing_time,
      if conflicts.contains(arrival) {
        "*"
      } else {
        " "
      },
      problem.planes[arrival.plane_id].cost_for_landing(arrival.landing_time) as i32,
      sep
    );
  }
}

fn run_sa(
  problem: &LandingProblem,
  solution: &Solution,
  sa_max_k: f64,
  alpha: f64,
  initial_temp: f64,
  bounded: bool,
) -> (Solution, Duration) {
  let n = problem.planes.len();
  let max_iterations = if bounded { 10 * n * n } else { usize::MAX };
  let start = time::Instant::now();
  let solution = simulated_annealing(
    problem,
    solution,
    max_iterations,
    alpha,
    (sa_max_k * n as f64) as usize,
    initial_temp,
  );
  (solution, start.elapsed())
}

fn main() {
  let args: Vec<String> = args().collect();
  match &args[..] {
    [_, run_type, sa_max_k, alpha, initial_temp, tail @ ..] => {
      let sa_max_k = sa_max_k.parse::<f64>().unwrap();
      let alpha = alpha.parse::<f64>().unwrap();
      let initial_temp = initial_temp.parse::<f64>().unwrap();
      match run_type.as_str() {
        "irace" => {
          let problem = LandingProblem::from_parser(parse_problem_data(&tail[0]).unwrap());
          let (solution, duration) = run_sa(
            &problem,
            &problem.initial_solution(),
            sa_max_k,
            alpha,
            initial_temp,
            true,
          );
          println!("{}", problem.cost(&solution));
          println!("{}", duration.as_secs_f64());
        }
        "eval-one" => {
          let [file_path, max_time] = tail else {
            panic!("Pass in file_path and max_time")
          };
          let max_time = max_time.parse::<f64>().unwrap();
          let problem = LandingProblem::from_parser(parse_problem_data(file_path).unwrap());
          let mut solution = problem.initial_solution();
          let mut duration = Duration::ZERO;
          display_solution(&problem, &solution);
          while duration.as_secs_f64() < max_time {
            let result = run_sa(&problem, &solution, sa_max_k, alpha, initial_temp, false);
            solution = result.0;
            duration += result.1;

            for _ in 0..10 {
              solution = problem.first_improvement_neighbor(&solution);
            }
          }
          display_solution(&problem, &solution);
        }
        "eval-all" => {
          let [folder_path, max_time] = tail else {
            panic!("Pass in folder_path max_time")
          };
          let max_time = max_time.parse::<f64>().unwrap();

          let mut files = std::fs::read_dir(folder_path)
            .unwrap()
            .try_collect::<Vec<_>>()
            .unwrap();
          files.sort_by_key(|f| {
            f.file_name()
              .into_string()
              .unwrap()
              .chars()
              .filter(|char| char.is_ascii_digit())
              .collect::<String>()
              .parse::<i32>()
              .unwrap()
          });
          let solutions = files.into_iter().map(|file| {
            let file_path = file.path().to_string_lossy().to_string();
            let problem = LandingProblem::from_parser(parse_problem_data(&file_path).unwrap());

            let mut solution = problem.initial_solution();
            let mut duration = Duration::ZERO;
            while duration.as_secs_f64() < max_time {
              let result = run_sa(&problem, &solution, sa_max_k, alpha, initial_temp, false);
              solution = result.0;
              duration += result.1;
              for _ in 0..10 {
                solution = problem.first_improvement_neighbor(&solution);
              }
            }
            (file, problem, solution)
          });

          for (file, problem, solution) in solutions {
            println!(
              "{}\t{}\t{}\t{}",
              file.file_name().into_string().unwrap(),
              problem.planes.len(),
              problem.landing_cost(&solution) as u64,
              problem.is_valid(&solution)
            );
          }
        }
        _ => panic!("Usage: cargo run <run_type> <sa_max_k> <alpha> <initial_temp> ..."),
      }
    }
    _ => panic!("Usage: cargo run <run_type> <sa_max_k> <alpha> <initial_temp> ..."),
  }

  // let problem = LandingProblem::from_parser(parse_problem_data(file_path).unwrap());
  // let n = problem.planes.len();
  // let mut solution = problem.initial_solution();

  // let max_iterations = if unbounded { usize::MAX } else { n * n * 100 };
  // let mut duration = Duration::ZERO;
  // loop {
  //   let before = time::Instant::now();
  //   solution = simulated_annealing(
  //     &problem,
  //     &solution,
  //     max_iterations,
  //     alpha,
  //     (sa_max_k * n as f64) as usize,
  //     initial_temp,
  //   );
  //   let after = time::Instant::now();
  //   duration += after.duration_since(before);
  //   if !unbounded || duration.as_secs_f32() > 2.0 {
  //     break;
  //   }
  // }
  // display_solution(&problem, &solution);
  // if !unbounded {
  //   println!("{}", problem.cost(&solution));
  //   println!("{}", duration.as_secs_f32());
  // } else {
  //   println!("Further improvements");
  //   for _ in 0..1000 {
  //     solution = problem.first_improvement_neighbor(&solution);
  //   }
  //   display_solution(&problem, &solution);
  // }
}

// fn main() {
//   let args: Vec<String> = args().collect();

//   let (file_path, max_iterations) = match &args[..] {
//     [_, file_path, max_iterations] => (file_path, max_iterations.parse::<usize>().unwrap()),
//     [_, file_path] => (file_path, 100),
//     _ => panic!("Usage: cargo run <input_file_path> <?max_iterations = 100>"),
//   };

//   let problem = LandingProblem::from_parser(parse_problem_data(file_path).unwrap());
//   let n = problem.planes.len();

//   let solution = problem.initial_solution();
//   display_solution(&problem, &solution);
//   let cost = problem.cost(&solution);

//   let initial_temp = initial_temperature(&problem, &solution, 2., 0.95, 10 * n, 10.);
//   println!("Initial temp: {}", initial_temp);
//   let solution = simulated_annealing(
//     &problem,
//     &solution,
//     usize::MAX,
//     0.99,
//     max_iterations * n,
//     10000.,
//   );
//   display_solution(&problem, &solution);

//   println!(
//     "Cost before: {}, Cost after: {}",
//     cost,
//     problem.cost(&solution)
//   );
// }
