#![allow(special_module_name)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;

use good_lp::{constraint, default_solver, variable, Expression, SolverModel};
use good_lp::{ProblemVariables, VariableDefinition};

mod lib;
use lib::{UnoptimizedProblem, Variable};

fn create_variables(
    variables: HashMap<String, Variable>,
) -> (
    ProblemVariables,
    Vec<String>,
    HashMap<String, variable::Variable>,
) {
    let mut variable_names = vec![];
    let mut problem_variables = ProblemVariables::new();
    for (name, variable) in variables {
        let mut variable_definition = VariableDefinition::new();
        if let Some(min) = variable.min {
            variable_definition = variable_definition.min(min as f64);
        } else if let Some(max) = variable.max {
            variable_definition = variable_definition.max(max as f64);
        }
        problem_variables.add(variable_definition);
        variable_names.push(name);
    }

    let variable_hashmap = {
        let mut variable_hashmap = HashMap::new();
        let mut counter = 0;
        for (var, _var_def) in problem_variables.iter_variables_with_def() {
            let var_name = variable_names[counter].clone();
            variable_hashmap.insert(var_name, var);
            counter += 1;
        }
        variable_hashmap
    };

    (problem_variables, variable_names, variable_hashmap)
}

fn create_constraints(
    problem_constraints: &Vec<lib::Constraint>,
    variable_hashmap: &HashMap<String, variable::Variable>,
) -> Vec<constraint::Constraint> {
    let mut constraints = vec![];
    for constraint in problem_constraints {
        let lhs = *variable_hashmap.get(&constraint.lhs).unwrap();
        let rhs = *variable_hashmap.get(&constraint.rhs).unwrap();

        let mut expression = Expression::from_other_affine(0.0);

        let postfix_stack = constraint
            .expr
            .split_whitespace()
            .collect::<VecDeque<&str>>();

        // slide over the postfix_stack 3 elements at a time
        for i in (0..postfix_stack.len() - 1).step_by(3) {
            let item1 = postfix_stack[i];
            let item2 = postfix_stack[i + 1];
            let item3 = postfix_stack[i + 2];

            let left_is_variable = variable_hashmap.contains_key(item1);

            expression = match item3 {
                "+" => {
                    if left_is_variable {
                        expression + (lhs + item2.parse::<f64>().unwrap())
                    } else {
                        expression + (item1.parse::<f64>().unwrap() + lhs)
                    }
                }
                "*" => {
                    if left_is_variable {
                        expression + (lhs * item2.parse::<f64>().unwrap())
                    } else {
                        expression + (item1.parse::<f64>().unwrap() * lhs)
                    }
                }
                "-" => {
                    if left_is_variable {
                        expression + (lhs - item2.parse::<f64>().unwrap())
                    } else {
                        expression + (item1.parse::<f64>().unwrap() - lhs)
                    }
                }
                _ => expression,
            };
        }

        // println!("{:?}", expression);

        let constraint = match constraint.r#type.as_str() {
            "leq" => good_lp::constraint!(expression <= rhs),
            "geq" => good_lp::constraint!(expression >= rhs),
            _ => good_lp::constraint!(lhs == rhs),
        };
        constraints.push(constraint);
    }
    constraints
}

fn create_expression(
    problem_expression: &String,
    variable_hashmap: &HashMap<String, variable::Variable>,
) -> Expression {
    let mut expression = Expression::from_other_affine(0.);
    let postfix_stack = problem_expression
        .split_whitespace()
        .collect::<VecDeque<&str>>();
    for i in (0..postfix_stack.len()).step_by(3) {
        let item1 = postfix_stack[i];
        let item2 = postfix_stack[i + 1];
        let item3 = postfix_stack[i + 2];

        let left_is_variable = variable_hashmap.contains_key(item1);

        expression = match item3 {
            "+" => {
                if left_is_variable {
                    let lhs = *variable_hashmap.get(item1).unwrap();
                    expression + (lhs + item2.parse::<f64>().unwrap())
                } else {
                    let rhs = *variable_hashmap.get(item2).unwrap();
                    expression + (item1.parse::<f64>().unwrap() + rhs)
                }
            }
            _ => expression,
        };
    }
    expression
}

fn main() -> Result<(), Box<dyn Error>> {
    let json_problem = r#"
{
    "variables": {
      "a": {"max": 1},
      "b": {"min": 2, "max": 10}
    },
    "objective": {
      "goal": "max",
      "expression": "10 b +"
    },
    "constraints": [
      {
        "expr": "a -2 *", 
        "type": "leq", 
        "lhs": "a", 
        "rhs": "b"
      },
      {
        "expr": "3 a +", 
        "type": "geq", 
        "lhs": "a", 
        "rhs": "b"
      }
    ]
  }
"#;
    // "expr": "(3-a*3)/(2+5)+1000>=b",
    // "expression": "10*(a-b/5)-b"

    let problem: UnoptimizedProblem = serde_json::from_str(json_problem)?;
    // println!("{:#?}", problem);

    // conver the json into variables and capture the variable names
    let (problem_variables, variable_names, variable_hashmap) = create_variables(problem.variables);
    println!("Vars {:#?}", variable_names);

    let expression = create_expression(&problem.objective.expression, &variable_hashmap);
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

    // solve the problem
    let solution = solution.solve()?;

    // print the solution
    let sol = solution.into_inner();
    println!("{:#?}", sol);

    Ok(())
}
