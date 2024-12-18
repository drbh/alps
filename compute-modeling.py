from dataclasses import dataclass
from typing import List, Dict, NamedTuple
import json

@dataclass
class GPU:
    memory: int  
    flops: float 
    latency: float

@dataclass
class Operation:
    size: int
    operations: int

def format_float(num: float) -> str:
    return f"{num:.10f}".rstrip('0').rstrip('.')

def compute_time(flops: float, operations: int) -> str:
    return format_float(operations / flops)

def transfer_time(size: int, latency: float) -> str:
    return format_float(size / (latency * 1e6)) 

def make_optimization_model(gpus: Dict[str, GPU], matrices: Dict[str, Operation]):
    def make_var(name: str, lb: int = 0, ub: int = 1):
        return {
            "name": name,
            "min": lb,
            "max": ub
        }
    
    # assignment and timing variables
    variables = {
        f"assign_{op}_GPU:{gpu}": make_var(f"assign_{op}_GPU:{gpu}")
        for op in matrices
        for gpu in gpus
    }
    
    variables.update({
        f"start_time_{op}": make_var(f"start_time_{op}", lb=0, ub=1000000)
        for op in list(matrices) + ["compute"]
    })
    
    # end time variables for each operation
    variables.update({
        f"end_time_{op}": make_var(f"end_time_{op}", lb=0, ub=1000000)
        for op in matrices
    })
    
    # objective function
    objective = {
        "goal": "min",
        "expression": "start_time_compute"
    }
    
    constraints = []
    
    # each operation must be assigned to exactly one GPU
    constraints.extend([
        {
            "name": f"assign_op_{op}",
            "expression": " + ".join(f"assign_{op}_GPU:{gpu}" for gpu in gpus) + " == 1"
        }
        for op in matrices
    ])
    
    # end time constraints for each operation
    M = 1000000  # constant
    for op in matrices:
        for gpu_id, gpu in gpus.items():
            compute_duration = compute_time(gpu.flops, matrices[op].operations)
            constraints.append({
                "name": f"end_time_{op}_GPU:{gpu_id}_lower",
                "expression": f"end_time_{op} >= start_time_{op} + {compute_duration} - {M} * (1 - assign_{op}_GPU:{gpu_id})"
            })
            constraints.append({
                "name": f"end_time_{op}_GPU:{gpu_id}_upper",
                "expression": f"end_time_{op} <= start_time_{op} + {compute_duration} + {M} * (1 - assign_{op}_GPU:{gpu_id})"
            })
    
    #final multiplication
    constraints.extend([
        {
            "name": f"dependency_{op}_compute",
            "expression": f"end_time_{op} + " + " + ".join(
                f"(assign_{op}_GPU:{gpu_id} * {transfer_time(matrices[op].size, gpu.latency)})"
                for gpu_id, gpu in gpus.items()
            ) + " <= start_time_compute"
        }
        for op in matrices
    ])
    
    # Non-overlapping constraints for operations on the same GPU
    for gpu_id in gpus:
        ops = list(matrices.keys())
        for i in range(len(ops)):
            for j in range(i + 1, len(ops)):
                op1, op2 = ops[i], ops[j]
                constraints.append({
                    "name": f"non_overlap_{op1}_{op2}_GPU:{gpu_id}",
                    "expression": f"end_time_{op1} <= start_time_{op2} + {M} * (2 - assign_{op1}_GPU:{gpu_id} - assign_{op2}_GPU:{gpu_id})"
                })
                constraints.append({
                    "name": f"non_overlap_{op2}_{op1}_GPU:{gpu_id}",
                    "expression": f"end_time_{op2} <= start_time_{op1} + {M} * (2 - assign_{op1}_GPU:{gpu_id} - assign_{op2}_GPU:{gpu_id})"
                })
    
    return {"variables": variables, "objective": objective, "constraints": constraints}

# run with 
# python compute-modeling.py > problems/compute-modeling.json && cargo run -- --input problems/compute-modeling.json | jq
if __name__ == "__main__":

    ## 2 equal GPUs
    # gpus = {
    #     "0": GPU(memory=256_000, flops=1, latency=32),
    #     "1": GPU(memory=256_000, flops=1, latency=32),
    # }
    
    ## 4 equal GPUs
    # gpus = {
    #     "0": GPU(memory=256_000, flops=1, latency=32),
    #     "1": GPU(memory=256_000, flops=1, latency=32),
    #     "2": GPU(memory=256_000, flops=1, latency=32),
    #     "3": GPU(memory=256_000, flops=1, latency=32),
    # }
    
    # 3 equal GPUs and 1 faster GPU
    gpus = {
        "0": GPU(memory=256_000, flops=1, latency=32),
        "1": GPU(memory=256_000, flops=1, latency=32),
        "2": GPU(memory=256_000, flops=1, latency=32),
        "3": GPU(memory=2*256_000, flops=2, latency=32),
    }
    
    matrices = {
        "A": Operation(size=256_000, operations=1_000_000),
        "B": Operation(size=256_000, operations=1_000_000)
    }
    
    print(json.dumps(make_optimization_model(gpus, matrices), indent=2))

    # Expected output for 3 equal GPUs and 1 faster GPU
    # {
    #     "assign_B_GPU:3": 0.625,
    #     "end_time_B": 125000.0,
    #     "assign_A_GPU:1": 0.125,
    #     "assign_B_GPU:1": 0.12500000011641532,
    #     "assign_A_GPU:3": 0.6250000000582077,
    #     "end_time_A": 125000.0,
    #     "assign_B_GPU:2": 0.125,
    #     "assign_B_GPU:0": 0.125,
    #     "assign_A_GPU:0": 0.125,
    #     "start_time_A": 0.0,
    #     "start_time_compute": 1000000.0,
    #     "start_time_B": 0.0,
    #     "assign_A_GPU:2": 0.125
    # }