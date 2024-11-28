use airplane_landing_scheduler::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use metaheuristics::{simulated_annealing, Problem};
use problem::LandingProblem;

fn criterion_benchmark(c: &mut Criterion) {
  let problem =
    LandingProblem::from_parser(parser::parse_problem_data("data/airland2.txt").unwrap());
  let s = problem.initial_solution();

  // c.bench_function("random neighbor", |b| {
  //   b.iter(|| problem.random_neighbor(&s))
  // });

  // c.bench_function("cost", |b| b.iter(|| problem.cost(&s)));

  // c.bench_function("conflict cost", |b| b.iter(|| problem.conflict_cost(&s)));

  // c.bench_function("landing cost", |b| b.iter(|| problem.landing_cost(&s)));

  // c.bench_function("simulated annealing", |b| {
  //   b.iter(|| simulated_annealing(&problem, &s, 1000, 0.99, 50, 1000.0))
  // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
