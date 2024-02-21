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
