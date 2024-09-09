import asyncio
import random
from master.config import *
from master.data import *
from master.utils import *
from collections import Counter
import time

cached_block_id = None
cached_weights = None
cached_max_weight_challenge_id = None
last_regular_draw_time = time.time()

async def run(state: State):
    while True:
        try:
            print(f"[job_manager] checking status of running jobs")
            await _execute(state)
            print(f"[job_manager] done")
        except Exception as e:
            print(f"[job_manager] error: {e}")
        finally:
            await asyncio.sleep(5)

async def _execute(state: State):
    global last_regular_draw_time
    block = state.query_data.block
    challenges = state.query_data.challenges
    algorithms = state.query_data.algorithms
    wasms = state.query_data.wasms
    available_jobs = state.available_jobs
    pending_benchmark_jobs = state.pending_benchmark_jobs

    challenge_map = dict(
        **{
            c.id: c.details.name
            for c in challenges.values()
        },
        **{
            c.details.name: c.id
            for c in challenges.values()
        }
    )
    algorithm_map = dict(
        **{
            a.id: a.details.name
            for a in algorithms.values()
        },
        **{
            f"{a.details.challenge_id}_{a.details.name}": a.id
            for a in algorithms.values()
        }
    )

    n = now()
    finished_jobs = [
        benchmark_id
        for benchmark_id, job in available_jobs.items()
        if n >= job.timestamps.end
    ]
    for benchmark_id in list(available_jobs): 
        job = available_jobs[benchmark_id]
        if n >= job.timestamps.end:
            print(f"[job_manager] job {benchmark_id} FINISHED")
            pending_benchmark_jobs[benchmark_id] = available_jobs.pop(benchmark_id)

    job_counter = Counter(
        f"{challenge_map[job.settings.challenge_id]}_{algorithm_map[job.settings.algorithm_id]}"
        for job in available_jobs.values()
    )
    print(f"[job_manager] jobs counter: {job_counter}")
    new_jobs = []

    weights = await _calibrate_challenges(state)
    
    for challenge_name, selected_algorithms in JOBS.items():
        challenge_id = challenge_map[challenge_name]
        for algorithm_name, job_config in selected_algorithms.items():
            algorithm_id = algorithm_map.get(f"{challenge_id}_{algorithm_name}", None)
            assert algorithm_id is not None, f"Algorithm '{algorithm_name}' for challenge '{challenge_name}' does not exist"
            
            custom_num_jobs = _get_num_jobs(weights, job_config, challenge_id)
            num_jobs = job_counter[f"{challenge_name}_{algorithm_name}"]
            if num_jobs >= custom_num_jobs:
                continue

            weight = await _get_weight(weights, job_config, challenge_id)
            duration = await _get_duration(weights, job_config, challenge_id)
                
            download_url = wasms[algorithm_id].details.download_url
            assert download_url is not None, f"Download URL for algorithm '{algorithm_id}' is None"
            timestamps = Timestamps(
                start=n,
                end=n + duration,
                submit=n + duration + job_config["wait_slave_duration"]
            )
            for _ in range(custom_num_jobs - num_jobs):
                
                if ACTIVATE_DIFFICULTIES_OPTIMIZATION:
                    if time.time() - last_regular_draw_time >= DIFFICULTIES_REGULAR_PERIOD:
                        difficulty = random.choice(challenges[challenge_id].block_data.qualifier_difficulties)
                        last_regular_draw_time = time.time()
                    else:
                        ratios = [d[0] / d[1] if d[1] != 0 else float('inf') for d in challenges[challenge_id].block_data.qualifier_difficulties]
                        ratios_array = np.array(ratios)
                        q1 = np.percentile(ratios_array, 25)
                        q3 = np.percentile(ratios_array, 75)

                        middle_difficulties = [d for d in challenges[challenge_id].block_data.qualifier_difficulties if q1 <= d[0] / d[1] <= q3]

                        difficulty = random.choice(middle_difficulties)                        
                else:
                    # FIXME use difficulty sampler
                    difficulty = random.choice(challenges[challenge_id].block_data.qualifier_difficulties)
                
                benchmark_id = f"{challenge_name}_{algorithm_name}_{difficulty[0]}_{difficulty[1]}_{now()}"
                print(f"[job_manager] job: {benchmark_id} CREATED with weight {weight}")
                job = Job(
                    download_url=download_url,
                    benchmark_id=benchmark_id,
                    settings=BenchmarkSettings(
                        algorithm_id=algorithm_id,
                        challenge_id=challenge_id,
                        difficulty=difficulty,
                        player_id=PLAYER_ID,
                        block_id=block.id
                    ),
                    solution_signature_threshold=challenges[challenge_id].block_data.solution_signature_threshold,
                    sampled_nonces=None,
                    wasm_vm_config=block.config["wasm_vm"],
                    weight=weight,
                    timestamps=timestamps,
                    solutions_data={}
                )
                new_jobs.append(job)

    available_jobs.update({job.benchmark_id: job for job in new_jobs})

async def _calibrate_challenges(state: State):
    global cached_block_id, cached_weights, cached_max_weight_challenge_id
    
    current_block_id = state.query_data.block.id  
    
    if cached_block_id == current_block_id and cached_weights is not None:
        return cached_weights

    solutions_by_challenge = await _get_solutions_by_challenge(state)    
    weights = {challenge_id: 0 for challenge_id in state.query_data.challenges}
    
    if AUTO_CALIBRATE_CHALLENGES:
        total_solutions = sum(solutions_by_challenge.values())
        
        if total_solutions > 0:
            min_proportion = None
            
            for challenge_id, num_solutions in solutions_by_challenge.items():
                challenge_proportion = num_solutions / total_solutions if total_solutions > 0 else 0
                weight = 1 / (challenge_proportion + 0.01)
                weights[challenge_id] = weight
                
                if min_proportion is None or weight < min_proportion:
                    min_proportion = weight
            
            if min_proportion > 0:
                for challenge_id in weights:
                    weights[challenge_id] = round(weights[challenge_id] / min_proportion)
            
            sorted_solutions = sorted(solutions_by_challenge.values(), reverse=True)
            max_solutions = sorted_solutions[0]
            second_max_solutions = sorted_solutions[1] if len(sorted_solutions) > 1 else 0
            
            challenges_with_max_solutions = [challenge_id for challenge_id, num_solutions in solutions_by_challenge.items() if num_solutions == max_solutions]
            
            proportion_max = max_solutions / total_solutions
            proportion_second = second_max_solutions / total_solutions
            
            challenge_with_most_solutions = None
            if len(challenges_with_max_solutions) == 1 and total_solutions > 0 and (proportion_max - proportion_second) > 0.2:
                challenge_with_most_solutions = challenges_with_max_solutions[0]
                weights[challenge_with_most_solutions] = 0
            
            max_weight = max(weights.values())  
            for challenge_id in weights:
                if weights[challenge_id] == 0 and challenge_id != challenge_with_most_solutions:
                    weights[challenge_id] = max_weight * 2

            
            sorted_weights = sorted(weights.items(), key=lambda item: item[1], reverse=True)
            cached_max_weight_challenge_id = sorted_weights[0][0]
    else:
        weights = {challenge_id: 1 for challenge_id in state.query_data.challenges}
    
    cached_block_id = current_block_id
    cached_weights = weights
    
    return weights

async def _get_solutions_by_challenge(state: State):
    solutions_by_challenge = {}
    for id, benchmark in state.query_data.benchmarks.items():
        challenge_id = benchmark.settings.challenge_id
        num_solutions = benchmark.details.num_solutions
        if challenge_id in solutions_by_challenge:
            solutions_by_challenge[challenge_id] += num_solutions
        else:
            solutions_by_challenge[challenge_id] = num_solutions
    return solutions_by_challenge

async def _get_weight(weights: dict[str, int], job_config: dict[str, float], challenge_id):
    weight = job_config["weight"]
    if AUTO_CALIBRATE_CHALLENGES:
        weight = round(weights.get(challenge_id, 1))
    return weight
    
async def _get_duration(weights: dict[str, int], job_config: dict[str, float], challenge_id):
    global cached_max_weight_challenge_id
    duration = job_config["benchmark_duration"]
    if AUTO_CALIBRATE_CHALLENGES and challenge_id == cached_max_weight_challenge_id and weights.get(challenge_id, 1) > 1:
        duration = round(duration * job_config["benchmark_duration_factor"] or 1)
    return duration
        
async def _get_num_jobs(weights: dict[str, int], job_config: dict[str, float], challenge_id):
    global cached_max_weight_challenge_id
    num_jobs = job_config["num_jobs"]
    if AUTO_CALIBRATE_CHALLENGES and challenge_id == cached_max_weight_challenge_id and weights.get(challenge_id, 1) > 1:
        num_jobs = round(num_jobs * job_config["num_jobs_factor"] or 1)
    return num_jobs
