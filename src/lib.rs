use good_lp::IntoAffineExpression;
use good_lp::{constraint, Expression, Variable as GoodVariable};
use good_lp::{default_solver, Solution, SolverModel};
use good_lp::{ProblemVariables, VariableDefinition};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::ops::Div;
use std::ops::Mul;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnoptimizedProblem {
    pub variables: HashMap<String, Variable>,
    pub objective: Objective,
    pub constraints: Vec<Constraint>,
}

// UnoptimizedProblem: From<&str>
impl From<&str> for UnoptimizedProblem {
    fn from(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub max: Option<i64>,
    pub min: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub goal: String,
    pub expression: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub name: String,
    pub expression: String,
}

pub fn tokenize(input_str: &str) -> Vec<InfixToken> {
    let mut tokens: Vec<InfixToken> = Vec::new();
    for ele in input_str.split_whitespace() {
        if ele == "+" {
            tokens.push(InfixToken::Operator(Operator::Add));
        } else if ele == "-" {
            tokens.push(InfixToken::Operator(Operator::Sub));
        } else if ele == "*" {
            tokens.push(InfixToken::Operator(Operator::Mul));
        } else if ele == "/" {
            tokens.push(InfixToken::Operator(Operator::Div));
        } else if ele == "(" {
            tokens.push(InfixToken::LeftParen);
        } else if ele == ")" {
            tokens.push(InfixToken::RightParen);
        } else if ele == "\n" {
            continue;
        }
        // check if it is a number or a variable
        else if ele.parse::<isize>().is_ok() {
            tokens.push(InfixToken::Operand(ele.parse::<isize>().unwrap()));
        } else {
            // add variable
            tokens.push(InfixToken::Variable(ele.to_string()));
        }
    }
    tokens
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operator {
    // `+`
    Add,
    // `-`
    Sub,
    // `*`
    Mul,
    // `/`
    Div,
}

#[derive(Debug, PartialEq)]
pub enum InfixToken {
    Operator(Operator),
    Operand(isize),
    Variable(String),
    LeftParen,
    RightParen,
}

#[derive(Debug, PartialEq)]
pub enum PostfixToken {
    Operator(Operator),
    Operand(isize),
    Variable(String),
}

/// Transforms an infix expression to a postfix expression.
///
/// If the infix expression is valid, outputs `Some(_)`;
/// otherwise, outputs `None`.
pub fn infix_to_postfix(tokens: &[InfixToken]) -> Option<Vec<PostfixToken>> {
    if check_valid(tokens) {
        let mut stack = Vec::new();
        let mut output_vec: Vec<PostfixToken> = Vec::new();
        for token in tokens {
            if let InfixToken::Operand(value) = token {
                output_vec.push(PostfixToken::Operand(*value));
            } else if let InfixToken::Variable(name) = token {
                output_vec.push(PostfixToken::Variable(name.to_string()));
            } else if let InfixToken::LeftParen = token {
                stack.push(token);
            } else if let InfixToken::RightParen = token {
                loop {
                    let popout_token = stack.pop().unwrap();
                    if let InfixToken::LeftParen = popout_token {
                        break;
                    } else if let InfixToken::Operator(operator) = popout_token {
                        output_vec.push(PostfixToken::Operator(*operator));
                    }
                }
            } else if let InfixToken::Operator(operator) = token {
                if stack.is_empty() {
                    stack.push(token);
                } else {
                    loop {
                        if !stack.is_empty() {
                            let top_token = stack[stack.len() - 1];

                            if let InfixToken::Operator(stack_operator) = top_token {
                                if *stack_operator == Operator::Add
                                    || *stack_operator == Operator::Sub
                                {
                                    if *operator == Operator::Sub || *operator == Operator::Add {
                                        if let InfixToken::Operator(operator4out) =
                                            stack.pop().unwrap()
                                        {
                                            output_vec.push(PostfixToken::Operator(*operator4out));
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                } else if *stack_operator == Operator::Mul
                                    || *stack_operator == Operator::Div
                                {
                                    if let InfixToken::Operator(operator4out) = stack.pop().unwrap()
                                    {
                                        output_vec.push(PostfixToken::Operator(*operator4out));
                                    } else {
                                        break;
                                    }
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    stack.push(token);
                }
            }
        }

        for _x in 0..stack.len() {
            if let InfixToken::Operator(operator) = stack.pop().unwrap() {
                output_vec.push(PostfixToken::Operator(*operator));
            }
        }

        Some(output_vec)
    } else {
        None
    }
}

pub fn check_valid(tokens: &[InfixToken]) -> bool {
    if !tokens.is_empty() {
        if let InfixToken::Operator(_) = tokens[tokens.len() - 1] {
            return false;
        }

        if let InfixToken::LeftParen = tokens[tokens.len() - 1] {
            return false;
        }

        if let InfixToken::RightParen = tokens[0] {
            return false;
        }

        if let InfixToken::Operator(_) = tokens[0] {
            return false;
        }

        let mut index = 0;
        for each_element in tokens {
            if index == 0 {
                index += 1;
                continue;
            } else {
                let previous = &tokens[index - 1];
                if let InfixToken::Operand(_) = each_element {
                    if let InfixToken::Operand(_) = previous {
                        return false;
                    }

                    if let InfixToken::RightParen = previous {
                        return false;
                    }
                }

                if let InfixToken::LeftParen = each_element {
                    if let InfixToken::Operand(_) = previous {
                        return false;
                    }

                    if let InfixToken::RightParen = previous {
                        return false;
                    }
                }

                if let InfixToken::Operator(_) = each_element {
                    if let InfixToken::LeftParen = previous {
                        return false;
                    }

                    if let InfixToken::Operator(_) = previous {
                        return false;
                    }
                }

                if let InfixToken::RightParen = each_element {
                    if let InfixToken::LeftParen = previous {
                        return false;
                    }

                    if let InfixToken::Operator(_) = previous {
                        return false;
                    }
                }
            }
            index += 1;
        }

        let mut num_paren = 0;
        for each_element in tokens {
            if let InfixToken::LeftParen = each_element {
                num_paren += 1;
            }
            if let InfixToken::RightParen = each_element {
                num_paren -= 1;
            }
        }

        if num_paren != 0 {
            return false;
        }

        true
    } else {
        false
    }
}

// add To trait to PostfixToken so it can be .to_string()
impl ToString for PostfixToken {
    fn to_string(&self) -> String {
        match self {
            PostfixToken::Operator(Operator::Add) => "+".to_string(),
            PostfixToken::Operator(Operator::Sub) => "-".to_string(),
            PostfixToken::Operator(Operator::Mul) => "*".to_string(),
            PostfixToken::Operator(Operator::Div) => "/".to_string(),
            PostfixToken::Operand(value) => value.to_string(),
            PostfixToken::Variable(name) => name.to_string(),
        }
    }
}

///

pub fn create_variables(
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
        for (counter, (var, _var_def)) in problem_variables.iter_variables_with_def().enumerate() {
            let var_name = variable_names[counter].clone();
            variable_hashmap.insert(var_name, var);
        }
        variable_hashmap
    };

    (problem_variables, variable_names, variable_hashmap)
}

pub fn parse_postfix_expression(
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
pub fn create_expression(
    problem_expression: &str,
    variable_hashmap: &HashMap<String, GoodVariable>,
) -> Expression {
    let postfix_tokens: Vec<&str> = problem_expression.split_whitespace().collect();
    parse_postfix_expression(postfix_tokens, variable_hashmap)
}

// similar to parsing the expression but we need to map the variable names to the actual variables
// and then create the constraints and apply the correct operator
// handles inequality along with operators
pub fn create_constraints(
    problem_constraints: &Vec<Constraint>,
    variable_hashmap: &HashMap<String, GoodVariable>,
) -> Vec<constraint::Constraint> {
    let mut constraints = vec![];
    for constraint in problem_constraints {
        let f = constraint.expression.clone();

        let inequalities = ["<=", ">=", "==", "<", ">"];

        // check that f contains an inequality
        let contains_inequality = inequalities.iter().any(|x| f.contains(x));
        if !contains_inequality {
            panic!("Constraint does not contain an inequality");
        }

        // find the inequality that is in the string
        let my_inequality = inequalities
            .iter()
            .find(|x| f.contains(*x))
            .expect("Failed to find inequality");

        // split on the inequality
        let split = f.split(my_inequality).collect::<Vec<&str>>();

        let lhs = split[0].trim();
        let rhs = split[1].trim();

        let lhs_postfix = tokenize(lhs);
        let rhs_postfix = tokenize(rhs);

        let lhs_expression = infix_to_postfix(&lhs_postfix).unwrap();
        let rhs_expression = infix_to_postfix(&rhs_postfix).unwrap();

        let lhs_postfix_string = lhs_expression
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let rhs_postfix_string = rhs_expression
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let lhs_postfix_tokens: Vec<&str> = lhs_postfix_string.split_whitespace().collect();
        let rhs_postfix_tokens: Vec<&str> = rhs_postfix_string.split_whitespace().collect();

        let lhs_expression = parse_postfix_expression(lhs_postfix_tokens, variable_hashmap);
        let rhs_expression = parse_postfix_expression(rhs_postfix_tokens, variable_hashmap);

        // use the my_inequality to create the constraint
        let constraint = match *my_inequality {
            "<=" => good_lp::constraint!(lhs_expression <= rhs_expression),
            ">=" => good_lp::constraint!(lhs_expression >= rhs_expression),
            "==" => good_lp::constraint!(lhs_expression == rhs_expression),
            // throw an error if the inequality is not supported
            _ => panic!("Unsupported inequality"),
        };

        constraints.push(constraint);
    }
    constraints
}

pub fn parse_objective_expression(objective: &str) -> String {
    let original_tokens = tokenize(objective);
    let result = infix_to_postfix(&original_tokens).unwrap();
    let postfix_string = result
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    postfix_string
}

pub fn solve(problem: UnoptimizedProblem) -> Result<String, Box<dyn Error>> {
    // let problem: UnoptimizedProblem = serde_json::from_str(&json_problem).unwrap();
    let (problem_variables, _variable_names, variable_hashmap) = create_variables(problem.variables);
    let parsed_expression = parse_objective_expression(&problem.objective.expression);
    let expression = create_expression(&parsed_expression, &variable_hashmap);
    let constraints = create_constraints(&problem.constraints, &variable_hashmap);
    let mut solution = problem_variables.maximise(expression).using(default_solver);
    for constraint in constraints {
        solution = solution.with(constraint);
    }
    let solution = solution.solve().unwrap();
    let mut values = vec![];
    for var in variable_hashmap.keys() {
        let value = variable_hashmap.get(var);
        let v = solution.value(*value.unwrap());
        values.push((var.clone(), v));
    }
    let sol = solution.into_inner();
    let response = format!("{:#?}", sol);
    Ok(response)
}
