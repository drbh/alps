use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnoptimizedProblem {
    pub variables: HashMap<String, Variable>,
    pub objective: Objective,
    pub constraints: Vec<Constraint>,
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
    pub expr: String,
    pub r#type: String,
    pub lhs: String,
    pub rhs: String,
}

pub fn tokenize(input_str: &String) -> Vec<InfixToken> {
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
