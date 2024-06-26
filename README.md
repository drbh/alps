# ⛰ alps

alps makes it easy to solve linear programming problems.

```bash
cargo run -- --input problems/bakery.json
#    Compiling alps v0.1.0 (/Users/drbh/Projects/alps)
#     Finished dev [unoptimized + debuginfo] target(s) in 0.23s
#      Running `target/debug/alps --input problems/bakery.json`
# Solution {
#     direction: Maximize,
#     num_vars: 2,
#     num_constraints: 5,
#     objective: 94.75,
# }
```

```bash
cargo test --package alps --bin alps -- tests --nocapture
#    Compiling alps v0.1.0 (/Users/drbh/Projects/alps)
#     Finished test [unoptimized + debuginfo] target(s) in 0.30s
#      Running unittests src/main.rs (target/debug/deps/alps-e86163ff8d944cc0)

# running 3 tests
# test tests::test_create_expression_simple ... ok
# test tests::test_create_constraint_simple ... ok
# test tests::test_create_expression ... ok

# test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

it can also be used a library

```rust
use alps::{solve, UnoptimizedProblem};

fn main() {
    // Load the problem from a file
    let file_str = std::fs::read_to_string("problems/bakery.json").expect("Failed to read the file");

    // Deserialize the problem
    let problem: UnoptimizedProblem = serde_json::from_str(&file_str).expect("Failed to deserialize the problem");

    // Solve the problem
    let solution = solve(&problem);

    // Print the solution
    println!("{:?}", solution);
}
```
