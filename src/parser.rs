use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug, Clone)]
pub struct Plane {
  pub earliest_landing: u32,
  pub target_landing: u32,
  pub latest_landing: u32,
  /// The penalty cost per unit of time for landing before the target time Ti
  pub penalty_before: f64,
  /// The penalty cost per unit of time for landing after the target time Ti
  pub penalty_after: f64,
  pub separation_times: Vec<u32>,
}

#[derive(Debug)]
pub struct ProblemData {
  pub num_planes: usize,
  pub planes: Vec<Plane>,
}

pub fn parse_problem_data(file_path: &str) -> io::Result<ProblemData> {
  let mut lines = io::BufReader::new(File::open(file_path)?).lines();

  // First line: number of planes and freeze time
  let first_line = lines.next().unwrap()?;
  let mut parts = first_line.split_whitespace();
  let num_planes: usize = parts.next().unwrap().parse().unwrap();
  let _freeze_time: u32 = parts.next().unwrap().parse().unwrap();

  let mut planes = Vec::with_capacity(num_planes);

  for _ in 0..num_planes {
    // Parse the plane's landing details
    let plane_line = lines.next().unwrap()?;
    let mut parts = plane_line.split_whitespace();
    let _appearance_time: u32 = parts.next().unwrap().parse().unwrap();
    let earliest_landing: u32 = parts.next().unwrap().parse().unwrap();
    let target_landing: u32 = parts.next().unwrap().parse().unwrap();
    let latest_landing: u32 = parts.next().unwrap().parse().unwrap();
    let penalty_before: f64 = parts.next().unwrap().parse().unwrap();
    let penalty_after: f64 = parts.next().unwrap().parse().unwrap();

    // Parse the separation times for this plane
    let mut separation_times = Vec::with_capacity(num_planes);
    while separation_times.len() < num_planes {
      let sep_line = lines.next().unwrap()?;
      separation_times.extend(
        sep_line
          .split_whitespace()
          .map(|s| s.parse::<u32>().unwrap_or_default()),
      );
      // println!("{} {}", separation_times.len(), num_planes);
    }

    // // It is the subsequent lines of the separation times
    // lines.next();

    planes.push(Plane {
      earliest_landing,
      target_landing,
      latest_landing,
      penalty_before,
      penalty_after,
      separation_times,
    });
  }

  // planes.sort_by(|a, b| a.target_landing.cmp(&b.target_landing));

  Ok(ProblemData { num_planes, planes })
}
