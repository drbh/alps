// test create_expression
#[cfg(test)]
use crate::lib::{
    //
    create_constraints,
    create_expression,
    create_variables,
    parse_objective_expression,
    UnoptimizedProblem,
};

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
              "expression": "10 + a - b"
            },
            "constraints": [
              {
                "name": "flour",
                "expression": "3 + a >= b"
              },
              {
                "name": "milk",
                "expression": "3 + a >= b"
              }
            ]
          }
        "#;
    let problem: UnoptimizedProblem = serde_json::from_str(json_problem).unwrap();
    let (_problem_variables, _variable_names, variable_hashmap) =
        create_variables(problem.variables);

    let parsed_expression = parse_objective_expression(&problem.objective.expression);
    let expression = create_expression(&parsed_expression, &variable_hashmap);
    let string_expression = format!("{:?}", expression);
    let acceptable_expressions = ["v1 + -1 v0 + 10".to_string(), "-1 v1 + v0 + 10".to_string()];

    let matched = acceptable_expressions
        .iter()
        .any(|x| x == &string_expression);

    assert!(matched);
}

#[test]
fn test_create_constraint_simple() {
    let json_problem = r#"
        {
            "variables": {
              "a": {"max": 1},
              "b": {"min": 2, "max": 10}
            },
            "objective": {
              "goal": "max",
              "expression": "a + ( -1 * b ) + 5"
            },
            "constraints": [
              {
                "name": "flour",
                "expression": "3 + a >= b"
              },
              {
                "name": "milk",
                "expression": "6 + a >= b"
              }
            ]
          }
        "#;
    let problem: UnoptimizedProblem = serde_json::from_str(json_problem).unwrap();
    let (_problem_variables, _variable_names, variable_hashmap) =
        create_variables(problem.variables);

    let parsed_expression = parse_objective_expression(&problem.objective.expression);
    let expression = create_expression(&parsed_expression, &variable_hashmap);
    let string_expression = format!("{:?}", expression);

    let constraints = create_constraints(&problem.constraints, &variable_hashmap);
    assert_eq!(constraints.len(), 2);

    let mut expected_constraints =
        vec!["-1 v1 + v0 <= 3".to_string(), "-1 v1 + v0 <= 6".to_string()];
    let mut other_expected_constraints =
        vec!["v1 + -1 v0 <= 3".to_string(), "v1 + -1 v0 <= 6".to_string()];

    // sort the vectors so that we can compare them
    expected_constraints.sort();
    other_expected_constraints.sort(); // TODO: integrate this into the test

    let acceptable_expressions_list = vec![
        expected_constraints.clone(),
        other_expected_constraints.clone(),
    ];

    let mut actual_constraints = constraints
        .iter()
        .map(|x| format!("{:?}", x))
        .collect::<Vec<String>>();

    actual_constraints.sort();

    let mut matches = vec![];
    for acceptable_expressions in acceptable_expressions_list {
        let matched = acceptable_expressions
            .iter()
            .any(|x| actual_constraints.contains(x));
        matches.push(matched);
    }

    assert!(matches.iter().any(|x| x == &true));

    // must match one of the acceptable expressions
    let acceptable_expressions = [
        "-1 v1 + v0 + 5".to_string(),
        "v1 + -1 v0 + 5".to_string(),
        "-1 v1 + v0 + 5".to_string(),
    ];

    let matched = acceptable_expressions
        .iter()
        .any(|x| x == &string_expression);

    assert!(matched);
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
              "expression": "a + ( -1 * b ) + 5"
            },
            "constraints": [
              {
                "name": "flour",
                "expression": "3 + a >= b"
              },
              {
                "name": "milk",
                "expression": "6 + a >= b"
              }
            ]
          }
        "#;
    let problem: UnoptimizedProblem = serde_json::from_str(json_problem).unwrap();
    let (_problem_variables, _variable_names, variable_hashmap) =
        create_variables(problem.variables);

    let parsed_expression = parse_objective_expression(&problem.objective.expression);
    let expression = create_expression(&parsed_expression, &variable_hashmap);
    let string_expression = format!("{:?}", expression);

    // a+2-b+3

    // must match one of the acceptable expressions
    let acceptable_expressions = [
        "v1 + -1 v0 + 5".to_string(),
        "v0 + -1 v1 + 5".to_string(),
        "-1 v1 + v0 + 5".to_string(),
    ];

    let matched = acceptable_expressions
        .iter()
        .any(|x| x == &string_expression);

    assert!(matched);
}
