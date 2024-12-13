use crate::metaheuristics::{self, initial_temperature, Problem};
use crate::parser;
use core::fmt;
use rand::prelude::Distribution;
use rand::{seq::SliceRandom, Rng};
use std::iter::zip;
use std::ops::IndexMut;
use std::{collections::HashSet, env::args, fmt::Debug, time};

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
pub struct Arrival {
  pub plane_id: ID,
  pub landing_time: u32,
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
pub type Solution = Vec<Arrival>;

/// Cost of a conflict of landing times between two planes per unit of time
const CONFLICT_PENALTY: f64 = 5000.0;

#[derive(Debug)]
pub struct LandingProblem {
  pub planes: Vec<Plane>,
  pub uniform: rand::distributions::Uniform<usize>,
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
      uniform: rand::distributions::Uniform::new(0, data.num_planes),
    }
  }

  pub fn conflicts<'a>(
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

  pub fn landing_cost(&self, solution: &Solution) -> f64 {
    solution
      .iter()
      .map(|arrival| self.planes[arrival.plane_id].cost_for_landing(arrival.landing_time))
      .sum()
  }

  pub fn conflict_cost(&self, solution: &Solution) -> f64 {
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
    let arrival_i = self.uniform.sample(&mut rng);

    let mut new_solution = solution.clone();
    let arrival = new_solution.index_mut(arrival_i);
    let plane = &self.planes[arrival.plane_id];
    let landing_time = rng.gen_range(plane.earliest_landing..=plane.latest_landing);
    arrival.landing_time = landing_time;
    new_solution.sort_by_key(|a| a.landing_time);
    new_solution
  }

  fn first_improvement_neighbor(&self, solution: &Solution) -> Solution {
    let mut rng = rand::thread_rng();
    let mut arrival_is = (0..solution.len()).collect::<Vec<_>>();
    arrival_is.shuffle(&mut rng);

    for arrival_i in arrival_is {
      let arrival = solution[arrival_i];
      let plane = &self.planes[arrival.plane_id];

      //  [0  1  2 ...  30].reverse()
      let towards_earliest = (plane.earliest_landing..=arrival.landing_time).rev();
      // [31 32 33 ... 100]
      let towards_latest = (arrival.landing_time..=plane.latest_landing);

      // [30 31 29 32 28 33 ... 0 100]
      let zigzag_times = zip(towards_earliest, towards_latest).flat_map(|(e, l)| [e, l]);

      for time in zigzag_times {
        let mut new_solution = solution.clone();
        new_solution[arrival_i].landing_time = time;
        new_solution.sort_by_key(|a| a.landing_time);

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
