use airplane_landing_scheduler::metaheuristics::{simulated_annealing, Problem};
use airplane_landing_scheduler::parser::parse_problem_data;
use airplane_landing_scheduler::problem::*;
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
fn main() {
  let args: Vec<String> = args().collect();

  let (file_path, sa_max_k, alpha, initial_temp, unbounded) = match &args[..] {
    [_, file_path, sa_max_k, alpha, initial_temp, unbounded] => (
      file_path,
      sa_max_k.parse::<f64>().unwrap(),
      alpha.parse::<f64>().unwrap(),
      initial_temp.parse::<f64>().unwrap(),
      unbounded.parse::<bool>().unwrap(),
    ),
    x => panic!("yo {:?}", x),
  };

  let problem = LandingProblem::from_parser(parse_problem_data(file_path).unwrap());
  let n = problem.planes.len();
  let solution = problem.initial_solution();

  let max_iterations = if unbounded { usize::MAX } else { n * n * 100 };

  let before = time::Instant::now();
  let solution = simulated_annealing(
    &problem,
    &solution,
    max_iterations,
    alpha,
    (sa_max_k * n as f64) as usize,
    initial_temp,
  );
  let after = time::Instant::now();

  display_solution(&problem, &solution);
  println!("{}", problem.cost(&solution));
  println!("{}", after.duration_since(before).as_secs_f32());
}

// fn main() {
//   let args: Vec<String> = args().collect();

//   let (file_path, max_iterations) = match &args[..] {
//     [_, file_path, max_iterations] => (file_path, max_iterations.parse::<usize>().unwrap()),
//     [_, file_path] => (file_path, 100),
//     _ => panic!("Usage: cargo run <input_file_path> <?max_iterations = 100>"),
//   };

//   let problem = LandingProblem::from_parser(parser::parse_problem_data(file_path).unwrap());
//   let n = problem.planes.len();

//   let solution = problem.initial_solution();
//   display_solution(&problem, &solution);
//   let cost = problem.cost(&solution);

//   let initial_temp =
//     metaheuristics::initial_temperature(&problem, &solution, 2., 0.95, 10 * n, 10.);
//   println!("Initial temp: {}", initial_temp);
//   let solution = metaheuristics::simulated_annealing(
//     &problem,
//     &solution,
//     0.99,
//     max_iterations * n,
//     initial_temp,
//   );
//   display_solution(&problem, &solution);

//   println!(
//     "Cost before: {}, Cost after: {}",
//     cost,
//     problem.cost(&solution)
//   );
// }
