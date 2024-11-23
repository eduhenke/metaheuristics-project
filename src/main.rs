#![allow(unused)]
#![feature(array_windows)]

use core::fmt;
use metaheuristics::{initial_temperature, Problem};
use rand::{seq::SliceRandom, Rng};
use std::{collections::HashSet, env::args, fmt::Debug};

mod metaheuristics;
mod parser;

type ID = usize;

#[derive(Debug)]
pub struct Plane {
  pub id: ID,
  pub earliest_landing: u32,
  pub target_landing: u32,
  pub latest_landing: u32,
  /// The penalty cost per unit of time for landing before the target time Ti
  pub penalty_before: f64,
  /// The penalty cost per unit of time for landing after the target time Ti
  pub penalty_after: f64,
  pub separation_times: Vec<u32>,
}

impl Plane {
  pub fn cost_for_landing(&self, landing_time: u32) -> f64 {
    if landing_time < self.target_landing {
      self.penalty_before * (self.target_landing - landing_time) as f64
    } else {
      self.penalty_after * (landing_time - self.target_landing) as f64
    }
  }
}

impl fmt::Display for Plane {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "(#{}, T={}<{}<{})",
      self.id, self.earliest_landing, self.target_landing, self.latest_landing
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Arrival {
  plane_id: ID,
  landing_time: u32,
}

impl Arrival {
  pub fn new(plane_id: ID, landing_time: u32) -> Self {
    Arrival {
      plane_id,
      landing_time,
    }
  }
}

/// Ordered list of arrivals
type Solution = Vec<Arrival>;

fn change_arrival(solution: &Solution, id: ID, landing_time: u32) -> Solution {
  let mut solution = solution.clone();
  solution
    .iter_mut()
    .find(|a| a.plane_id == id)
    .unwrap()
    .landing_time = landing_time;
  solution.sort_by(|a, b| a.landing_time.cmp(&b.landing_time));
  solution
}

/// Cost of a conflict of landing times between two planes per unit of time
const CONFLICT_PENALTY: f64 = 1000.0;

#[derive(Debug)]
struct LandingProblem {
  pub planes: Vec<Plane>,
}

impl LandingProblem {
  pub fn from_parser(data: parser::ProblemData) -> Self {
    assert_eq!(data.num_planes, data.planes.len());
    LandingProblem {
      planes: data
        .planes
        .into_iter()
        .enumerate()
        .map(|(id, p)| Plane {
          id,
          earliest_landing: p.earliest_landing,
          target_landing: p.target_landing,
          latest_landing: p.latest_landing,
          penalty_before: p.penalty_before,
          penalty_after: p.penalty_after,
          separation_times: p.separation_times,
        })
        .collect(),
    }
  }

  fn conflicts<'a>(
    &'a self,
    solution: &'a Solution,
  ) -> impl Iterator<Item = (Arrival, Arrival, u32)> + 'a {
    solution.array_windows::<2>().filter_map(|[a, b]| {
      let max_landing_time = b.landing_time - self.separation_time_between(a.plane_id, b.plane_id);

      if a.landing_time > max_landing_time {
        Some((*a, *b, a.landing_time - max_landing_time))
      } else {
        None
      }
    })
  }

  pub fn is_valid(&self, solution: &Solution) -> bool {
    self.conflicts(solution).next().is_none()
  }

  pub fn separation_time_between(&self, a: ID, b: ID) -> u32 {
    self.planes[a].separation_times[b]
  }

  fn landing_cost(&self, solution: &Solution) -> f64 {
    solution
      .iter()
      .map(|arrival| self.planes[arrival.plane_id].cost_for_landing(arrival.landing_time))
      .sum()
  }

  fn conflict_cost(&self, solution: &Solution) -> f64 {
    self
      .conflicts(solution)
      .map(|(_, _, conflict_duration)| CONFLICT_PENALTY * conflict_duration as f64)
      .sum()
  }
}

impl metaheuristics::Problem<Solution> for LandingProblem {
  fn initial_solution(&self) -> Solution {
    let mut s: Vec<_> = self
      .planes
      .iter()
      .map(|p| Arrival::new(p.id, p.target_landing))
      .collect();
    s.sort_by(|a, b| a.landing_time.cmp(&b.landing_time));
    s
  }

  fn random_neighbor(&self, solution: &Solution) -> Solution {
    let mut rng = rand::thread_rng();
    let id = rng.gen_range(0..self.planes.len());

    let plane = &self.planes[id];
    let landing_time = rng.gen_range(plane.earliest_landing..=plane.latest_landing);
    change_arrival(solution, id, landing_time)
  }

  fn first_improvement_neighbor(&self, solution: &Solution) -> Solution {
    let mut rng = rand::thread_rng();
    let mut ids = solution.iter().map(|a| a.plane_id).collect::<Vec<_>>();
    ids.shuffle(&mut rng);

    for id in ids {
      let plane = &self.planes[id];
      for time in plane.earliest_landing..=plane.latest_landing {
        let new_solution = change_arrival(solution, id, time);
        if self.cost(&new_solution) < self.cost(solution) {
          return new_solution;
        }
      }
    }
    solution.clone()
  }

  fn cost(&self, solution: &Solution) -> f64 {
    self.landing_cost(solution) + self.conflict_cost(solution)
  }
}

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

  let (file_path, max_iterations) = match &args[..] {
    [_, file_path, max_iterations] => (file_path, max_iterations.parse::<usize>().unwrap()),
    [_, file_path] => (file_path, 100),
    _ => panic!("Usage: cargo run <input_file_path> <?max_iterations = 100>"),
  };

  let problem = LandingProblem::from_parser(parser::parse_problem_data(file_path).unwrap());
  let n = problem.planes.len();

  let solution = problem.initial_solution();
  display_solution(&problem, &solution);
  let cost = problem.cost(&solution);

  let initial_temp =
    metaheuristics::initial_temperature(&problem, &solution, 2., 0.95, 10 * n, 10.);
  println!("Initial temp: {}", initial_temp);
  let solution = metaheuristics::simulated_annealing(
    &problem,
    &solution,
    0.99,
    max_iterations * n,
    initial_temp,
  );
  display_solution(&problem, &solution);

  println!(
    "Cost before: {}, Cost after: {}",
    cost,
    problem.cost(&solution)
  );
}
