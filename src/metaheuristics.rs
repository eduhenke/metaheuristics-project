use rand::Rng;

pub trait Problem<S> {
  fn initial_solution(&self) -> S;
  fn random_neighbor(&self, solution: &S) -> S;
  fn first_improvement_neighbor(&self, solution: &S) -> S;
  fn cost(&self, solution: &S) -> f64;

  fn best_solution(&self, solutions: Vec<S>) -> S {
    solutions
      .into_iter()
      .min_by(|a, b| self.cost(a).partial_cmp(&self.cost(b)).unwrap())
      .unwrap()
  }

  fn shake(&self, mut solution: S, intensity: i32) -> S {
    for _ in 0..intensity {
      solution = self.random_neighbor(&solution);
    }
    solution
  }
}

const NEIGHBORS: usize = 5;

pub fn hill_climb<S, P: Problem<S>>(problem: &P, mut s: S, gas: usize) -> S {
  for _ in 0..gas {
    let mut neighbors: Vec<_> = (0..NEIGHBORS)
      .map(|_| problem.first_improvement_neighbor(&s))
      .collect();
    neighbors.push(s);
    s = problem.best_solution(neighbors);
  }
  s
}

pub fn ils<S: Clone, P: Problem<S>>(problem: &P, ils_gas: usize, climb_gas: usize) -> S {
  let mut s = problem.initial_solution();
  s = hill_climb(problem, s, climb_gas);
  let mut i = 0;
  let mut best_i = 0;
  let mut intensity = 1;
  while i - best_i < ils_gas {
    i += 1;
    let s_shake = problem.shake(s.clone(), intensity);
    let s_shake = hill_climb(problem, s_shake, climb_gas);
    if problem.cost(&s_shake) < problem.cost(&s) {
      best_i = i;
      intensity = 1;
      s = s_shake;
    } else {
      intensity += 1;
    }
  }
  s
}

type Temperature = f64;
pub fn initial_temperature<S: Clone, P: Problem<S>>(
  problem: &P,
  s: &S,
  beta: f64,
  gamma: f64,
  sa_gas: usize,
  mut temp: Temperature,
) -> Temperature {
  loop {
    let mut accepted = 0;
    for _ in 0..sa_gas {
      let neighbor = problem.random_neighbor(s);
      let delta = problem.cost(&neighbor) - problem.cost(s);
      if delta < 0.0 {
        accepted += 1;
      } else {
        let x = rand::thread_rng().gen_range(0.0..=1.0);
        if x < (-delta / temp).exp() {
          accepted += 1;
        }
      }
    }
    if (accepted as f64) > (gamma * sa_gas as f64) {
      break;
    } else {
      temp *= beta;
    }
  }
  temp
}

pub fn simulated_annealing<S: Clone, P: Problem<S>>(
  problem: &P,
  s: &S,
  max_iterations: usize,
  alpha: f64,
  sa_max: usize,
  mut temp: Temperature,
) -> S {
  let mut s = s.clone();
  let mut best_s = s.clone();
  let mut global_iter = 0;

  while temp > 0.1 {
    for _ in 0..sa_max {
      global_iter += 1;
      if global_iter > max_iterations {
        return best_s;
      }
      let neighbor = problem.random_neighbor(&s);
      let delta = problem.cost(&neighbor) - problem.cost(&s);
      if delta < 0.0 {
        s = neighbor;
        if problem.cost(&s) < problem.cost(&best_s) {
          best_s = s.clone();
        }
      } else {
        let x = rand::thread_rng().gen_range(0.0..=1.0);
        if x < (-delta / temp).exp() {
          s = neighbor;
        }
      }
    }
    temp *= alpha;
  }
  best_s
}
