#![allow(special_module_name)]
use good_lp::{default_solver, SolverModel};
use std::error::Error;
mod lib;
mod tests;
use clap::Parser;
use lib::{
    //
    create_constraints,
    create_expression,
    create_variables,
    parse_objective_expression,
    UnoptimizedProblem,
};

/// App Configuration
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // The name of the file to read from
    #[clap(short, long)]
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let json_problem = std::fs::read_to_string(args.input)?;

    let problem: UnoptimizedProblem = serde_json::from_str(&json_problem)?;
    println!("{:#?}", problem);

    // conver the json into variables and capture the variable names
    let (problem_variables, variable_names, variable_hashmap) = create_variables(problem.variables);
    println!("Vars {:#?}", variable_names);

    let parsed_expression = parse_objective_expression(&problem.objective.expression);

    let expression = create_expression(&parsed_expression, &variable_hashmap);
    println!("Expression {:?}", expression);

    // create the constraints
    let constraints = create_constraints(&problem.constraints, &variable_hashmap);
    println!("Constraints {:#?}", constraints);

    // now actually solve the problem

    let mut solution = problem_variables.maximise(expression).using(default_solver);

    // add constraint to the problem
    for constraint in constraints {
        solution = solution.with(constraint);
    }

    use good_lp::Solution;

    // solve the problem
    let solution = solution.solve()?;

    for var in variable_hashmap.keys() {
        let value = variable_hashmap.get(var);
        let v = solution.value(*value.unwrap());
        println!("{}: {}", var, v);
    }

    // print the solution
    let sol = solution.into_inner();
    println!("{:#?}", sol);

    Ok(())
}
