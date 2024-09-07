import asyncio
from signal import SIGINT, SIGTERM
from typing import Dict, Any
from master.data import *
from master import (
    data_fetcher, 
    recomputer, 
    job_manager, 
    webserver, 
    benchmark_submitter,
    proof_submitter
)

async def main():
    try:
        state = State(
            query_data=await data_fetcher._execute(),
            available_jobs={},
            pending_benchmark_jobs={},
            pending_proof_jobs={},
            submitted_proof_ids=set(),
            difficulty_samplers={}
        )
        await asyncio.gather(*[
            job_manager.run(state),
            data_fetcher.run(state),
            webserver.run(state),
            recomputer.run(state),
            benchmark_submitter.run(state),
            proof_submitter.run(state)
        ])
    except asyncio.CancelledError:
        do_cleanup()

if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    coro = main()
    try:
        loop.run_until_complete(coro)
    except KeyboardInterrupt:
        print("Received exit, exiting")