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
}

const NEIGHBORS: usize = 5;

pub fn hill_climb<S, P: Problem<S>>(problem: &P, max_iterations: usize) -> S {
  let mut current_solution = problem.initial_solution();
  for _ in 0..max_iterations {
    let mut neighbors: Vec<_> = (0..NEIGHBORS)
      .map(|_| problem.first_improvement_neighbor(&current_solution))
      .collect();
    neighbors.push(current_solution);
    current_solution = problem.best_solution(neighbors);
  }
  current_solution
}
