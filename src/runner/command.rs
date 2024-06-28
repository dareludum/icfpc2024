use argh::FromArgs;
use serde::{Deserialize, Serialize};

#[derive(FromArgs, PartialEq, Debug)]
/// Evaluate a program
#[argh(subcommand, name = "solve")]
pub struct SolveCommand {
    #[argh(positional)]
    pub problem_name: String,
    #[argh(positional)]
    pub solver_spec: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SolutionMetadata {
    solver_spec: String,
    score: u64,
}

impl SolveCommand {
    pub fn run(&self) {
        let solver = super::create_solver(&self.solver_spec);
        println!("Solver: {}", solver.name());

        let current_dir = std::env::current_dir().expect("Failed to get the current directory");
        let problem_dir = current_dir.join("problems").join(&self.problem_name);
        let current_solutions_dir = current_dir
            .join("solutions")
            .join("current")
            .join(&self.problem_name);
        let best_solutions_dir = current_dir
            .join("solutions")
            .join("best")
            .join(&self.problem_name);

        if !problem_dir.exists() {
            eprintln!("Problem path not found: {}", problem_dir.display());
            std::process::exit(1);
        }

        // Make sure all paths exist
        std::fs::create_dir_all(&current_solutions_dir)
            .expect("Failed to create the current solution directory");
        std::fs::create_dir_all(&best_solutions_dir)
            .expect("Failed to create the best solution directory");

        // Load all problems from the problem directory
        let problems = std::fs::read_dir(problem_dir)
            .expect("Failed to read the problem directory")
            .map(|entry| entry.expect("Failed to read a problem directory entry"))
            .map(|entry| {
                let problem_name = entry.file_name().to_string_lossy().to_string();
                let problem_path = entry.path();
                super::Problem::new(problem_path, problem_name)
            })
            .collect::<Vec<_>>();

        // Solve all problems with the given solver
        for problem in problems {
            println!("Solving problem: {}", problem.name);

            let mut best_solution_path = best_solutions_dir.join(&problem.name);
            best_solution_path.set_extension("icfp");
            let mut current_solution_path = current_solutions_dir.join(&problem.name);
            current_solution_path.set_extension("icfp");

            let mut solver = solver.clone();
            solver.initialize(problem.clone(), None);
            let solution = solver.solve();

            solution.save(&current_solution_path);
            let current_metadata = SolutionMetadata {
                solver_spec: solver.name(),
                score: solution.score(),
            };
            std::fs::write(
                current_solution_path.with_extension("meta"),
                serde_json::to_string(&current_metadata).unwrap(),
            )
            .expect("Failed to write the current solution metadata");

            let mut new_best = false;
            if best_solution_path.exists() {
                let metadata_file = best_solution_path.with_extension("meta");
                let best_metadata: SolutionMetadata = serde_json::from_str(
                    &std::fs::read_to_string(metadata_file)
                        .expect("Failed to read the metadata file"),
                )
                .unwrap();

                println!(
                    "Best solution: {} (solver: {})",
                    best_metadata.score, best_metadata.solver_spec
                );

                if solution.score() < best_metadata.score {
                    println!("New best solution: {}", current_metadata.score);
                    println!("!!! WE ARE WINNING SON !!!");

                    new_best = true;
                } else {
                    println!(
                        "Current solution (not better): {} >= {}",
                        current_metadata.score, best_metadata.score
                    );
                }
            } else {
                println!("First solution: {}", current_metadata.score);

                new_best = true;
            }

            if new_best {
                std::fs::copy(&current_solution_path, &best_solution_path)
                    .expect("Failed to copy the current solution to the best solution");
                std::fs::copy(
                    current_solution_path.with_extension("meta"),
                    best_solution_path.with_extension("meta"),
                )
                .expect("Failed to copy the current solution metadata to the best solution");
            }
        }
    }
}
