import asyncio
from quart import Quart, jsonify, request
from datetime import datetime
from hypercorn.config import Config
from hypercorn.asyncio import serve
from master.data import *
from master.config import *

async def run(state: State):
    app = Quart("tig-benchmarker")

    @app.route('/jobs', methods=['GET'])
    async def get_jobs():
        print(f"[webserver] slave {request.remote_addr} - fetched jobs")
        return jsonify(state.available_jobs)

    @app.route('/solutions_data/<benchmark_id>', methods=['POST'])
    async def submit_solution(benchmark_id):
        if not request.is_json:
            return "Request must be JSON", 400
        
        slave_addr = request.remote_addr
        try:
            solutions_data = await request.get_json()
            solutions_data = {int(nonce): SolutionData(**d) for nonce, d in solutions_data.items()}
        except Exception as e:
            print(f"[webserver] slave {slave_addr} - error parsing solution data: {e}")
            return "Invalid solution data", 400
        
        print(f"[webserver] slave {slave_addr} - posted {len(solutions_data)} solutions for job {benchmark_id}")
        job = None
        if benchmark_id in state.available_jobs:
            job = state.available_jobs[benchmark_id]
        elif benchmark_id in state.pending_benchmark_jobs:
            job = pending_benchmark_jobs[benchmark_id]
        else:
            print(f"[webserver] error job {benchmark_id} not found")
            return "Job not found", 404
        job.solutions_data.update(solutions_data)
        state.difficulty_samplers[job.settings.challenge_id].update_with_solutions(
            difficulty=job.settings.difficulty,
            num_solutions=len(solutions_data)                
        )
        return "OK", 200

    config = Config()
    config.bind = [f"0.0.0.0:{PORT}"]
    await serve(app, config)