#![allow(special_module_name)]

use std::collections::HashMap;

use std::error::Error;
use std::ops::Div;
use std::ops::Mul;

use good_lp::IntoAffineExpression;
use good_lp::{
    constraint, default_solver, Expression, ProblemVariables, SolverModel,
    Variable as GoodVariable, VariableDefinition,
};

mod lib;
use lib::{UnoptimizedProblem, Variable};

fn create_variables(
    variables: HashMap<String, Variable>,
) -> (ProblemVariables, Vec<String>, HashMap<String, GoodVariable>) {
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

fn parse_postfix_expression(
    postfix_tokens: Vec<&str>,
    variable_hashmap: &HashMap<String, GoodVariable>,
) -> Expression {
    let mut stack: Vec<Expression> = Vec::new();

    for token in postfix_tokens {
        match token {
            "+" | "-" | "*" | "/" => {
                let right = stack.pop().expect("Stack underflow");
                let left = stack.pop().expect("Stack underflow");

                let result = match token {
                    "+" => left + right,
                    "-" => left - right,
                    "*" => {
                        let left_coeffs = left.clone().linear_coefficients();
                        let left_constant = left.clone().constant();
                        let right_constant = right.clone().constant();

                        if left_coeffs.len() == 0 {
                            right.mul(left_constant)
                        } else {
                            left.mul(right_constant)
                        }
                    }
                    "/" => {
                        let left_coeffs = left.clone().linear_coefficients();
                        let left_constant = left.clone().constant();
                        let right_constant = right.clone().constant();

                        if left_coeffs.len() == 0 {
                            right.div(left_constant)
                        } else {
                            left.div(right_constant)
                        }
                    }
                    _ => panic!("Unsupported operator"),
                };
                stack.push(result);
            }
            _ => {
                let operand = if let Some(var) = variable_hashmap.get(token) {
                    Expression::from(*var)
                } else {
                    Expression::from(token.parse::<f64>().expect("Failed to parse float"))
                };
                stack.push(operand);
            }
        }
    }

    stack.pop().expect("No expression was created")
}

// parse postfix expressions into good_lp expressions
// "expression": "a 2 * b + 3 +"
// only handles ADD, SUB, MUL, DIV
fn create_expression(
    problem_expression: &str,
    variable_hashmap: &HashMap<String, GoodVariable>,
) -> Expression {
    let postfix_tokens: Vec<&str> = problem_expression.split_whitespace().collect();
    parse_postfix_expression(postfix_tokens, variable_hashmap)
}

// similar to parsing the expression but we need to map the variable names to the actual variables
// and then create the constraints and apply the correct operator
// handles inequality along with operators
fn create_constraints(
    problem_constraints: &Vec<lib::Constraint>,
    variable_hashmap: &HashMap<String, GoodVariable>,
) -> Vec<constraint::Constraint> {
    let mut constraints = vec![];
    for constraint in problem_constraints {
        let lhs = *variable_hashmap.get(&constraint.lhs).unwrap();
        let rhs = *variable_hashmap.get(&constraint.rhs).unwrap();

        let postfix_tokens: Vec<&str> = constraint.expr.split_whitespace().collect();
        let expression = parse_postfix_expression(postfix_tokens, variable_hashmap);

        let constraint = match constraint.r#type.as_str() {
            "leq" => good_lp::constraint!(expression <= rhs),
            "geq" => good_lp::constraint!(expression >= rhs),
            _ => good_lp::constraint!(lhs == rhs),
        };
        constraints.push(constraint);
    }
    constraints
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
      "expression": "3 a * b + 3 *"
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

// test create_expression
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_expression_simple() {
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
        let problem: UnoptimizedProblem = serde_json::from_str(json_problem).unwrap();
        let (_problem_variables, _variable_names, variable_hashmap) =
            create_variables(problem.variables);
        let expression = create_expression(&problem.objective.expression, &variable_hashmap);
        println!("Expression {:?}", expression);
        let string_expression = format!("{:?}", expression);
        assert_eq!(string_expression, "v1 + 10".to_string());
    }

    #[test]
    fn test_create_expression() {
        let json_problem = r#"
        {
            "variables": {
              "a": {"max": 1},
              "b": {"min": 2, "max": 10}
            },
            "objective": {
              "goal": "max",
              "expression": "a 2 + b - 3 +"
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
        let problem: UnoptimizedProblem = serde_json::from_str(json_problem).unwrap();
        let (_problem_variables, _variable_names, variable_hashmap) =
            create_variables(problem.variables);
        let expression = create_expression(&problem.objective.expression, &variable_hashmap);
        println!("Expression {:?}", expression);
        let string_expression = format!("{:?}", expression);

        println!("String expression {:?}", &string_expression.clone());

        // a+2-b+3

        // must match one of the acceptable expressions
        let acceptable_expressions = vec![
            "v1 + -1 v0 + 5".to_string(),
            "v0 + -1 v1 + 5".to_string(),
            "-1 v1 + v0 + 5".to_string(),
        ];

        let matched = acceptable_expressions
            .iter()
            .any(|x| x == &string_expression);

        assert!(matched);
    }
}
