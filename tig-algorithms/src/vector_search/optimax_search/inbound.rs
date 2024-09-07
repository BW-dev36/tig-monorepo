/*!
Copyright 2024 bw-dev36

Licensed under the TIG Inbound Game License v1.0 or (at your option) any later
version (the "License"); you may not use this file except in compliance with the
License. You may obtain a copy of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

use anyhow::Ok;
use std::time::Instant;
use tig_challenges::vector_search::*;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

struct KDNode<'a> {
    point: &'a [f32],
    left: Option<Box<KDNode<'a>>>,
    right: Option<Box<KDNode<'a>>>,
    index: usize,
}

impl<'a> KDNode<'a> {
    fn new(point: &'a [f32], index: usize) -> Self {
        KDNode {
            point,
            left: None,
            right: None,
            index,
        }
    }
}
fn quickselect_by<F>(arr: &mut [(&[f32], usize)], k: usize, compare: &F)
where
    F: Fn(&(&[f32], usize), &(&[f32], usize)) -> Ordering,
{
    if arr.len() <= 1 {
        return;
    }

    let pivot_index = partition(arr, compare);
    if k < pivot_index {
        quickselect_by(&mut arr[..pivot_index], k, compare);
    } else if k > pivot_index {
        quickselect_by(&mut arr[pivot_index + 1..], k - pivot_index - 1, compare);
    }
}

fn partition<F>(arr: &mut [(&[f32], usize)], compare: &F) -> usize
where
    F: Fn(&(&[f32], usize), &(&[f32], usize)) -> Ordering,
{
    let pivot_index = arr.len() >> 1;
    arr.swap(pivot_index, arr.len() - 1);

    let mut store_index = 0;
    for i in 0..arr.len() - 1 {
        if compare(&arr[i], &arr[arr.len() - 1]) == Ordering::Less {
            arr.swap(i, store_index);
            store_index += 1;
        }
    }
    arr.swap(store_index, arr.len() - 1);
    store_index
}

fn build_kd_tree<'a>(points: &mut [(&'a [f32], usize)]) -> Option<Box<KDNode<'a>>> {
    if points.is_empty() {
        return None;
    }

    const NUM_DIMENSIONS: usize = 250;
    let mut stack: Vec<(usize, usize, usize, Option<*mut KDNode<'a>>, bool)> = Vec::new();
    let mut root: Option<Box<KDNode<'a>>> = None;

    stack.push((0, points.len(), 0, None, false));

    while let Some((start, end, depth, parent_ptr, is_left)) = stack.pop() {
        if start >= end {
            continue;
        }

        let axis = depth % NUM_DIMENSIONS;
        let median = (start + end) / 2;

        // Need to implement QuicSort or better QuickSelect
        quickselect_by(&mut points[start..end], median - start, &|a, b| {
            a.0[axis].partial_cmp(&b.0[axis]).unwrap()
        });

        let (median_point, median_index) = points[median];
        let mut new_node = Box::new(KDNode::new(median_point, median_index));
        let new_node_ptr: *mut KDNode = &mut *new_node;

        if let Some(parent_ptr) = parent_ptr {
            unsafe {
                if is_left {
                    (*parent_ptr).left = Some(new_node);
                } else {
                    (*parent_ptr).right = Some(new_node);
                }
            }
        } else {
            root = Some(new_node);
        }

        stack.push((median + 1, end, depth + 1, Some(new_node_ptr), false));
        stack.push((start, median, depth + 1, Some(new_node_ptr), true));
    }

    root
}

fn squared_euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum
}

#[inline(always)]
fn early_stopping_distance(a: &[f32], b: &[f32], current_min: f32) -> f32 {
    let mut sum = 0.0;
    let mut i = 0;

    //Try to unroll for simd vectorization
    while i + 3 < a.len() {
        let diff0 = a[i] - b[i];
        let diff1 = a[i + 1] - b[i + 1];
        let diff2 = a[i + 2] - b[i + 2];
        let diff3 = a[i + 3] - b[i + 3];

        sum += diff0 * diff0 + diff1 * diff1 + diff2 * diff2 + diff3 * diff3;

        if sum > current_min {
            return f32::MAX;
        }

        i += 4;
    }

    while i < a.len() {
        let diff = a[i] - b[i];
        sum += diff * diff;

        if sum > current_min {
            return f32::MAX;
        }

        i += 1;
    }

    sum
}

fn nearest_neighbor_search<'a>(
    root: &Option<Box<KDNode<'a>>>,
    target: &[f32],
    best: &mut (f32, Option<usize>),
) {
    let num_dimensions = target.len();
    let mut stack = Vec::with_capacity(64);

    if let Some(node) = root {
        stack.push((node.as_ref(), 0));
    }

    while let Some((node, depth)) = stack.pop() {
        let axis = depth % num_dimensions;
        let dist = early_stopping_distance(&node.point, target, best.0);

        if dist < best.0 {
            best.0 = dist;
            best.1 = Some(node.index);
        }

        let diff = target[axis] - node.point[axis];
        let sqr_diff = diff * diff;

        if sqr_diff < best.0 {
            if let Some(farther_node) = if diff < 0.0 { &node.right } else { &node.left } {
                stack.push((farther_node.as_ref(), depth + 1));
            }
        }

        if let Some(nearer_node) = if diff < 0.0 { &node.left } else { &node.right } {
            stack.push((nearer_node.as_ref(), depth + 1));
        }
    }
}

fn calculate_mean_vector(vectors: &[&[f32]]) -> Vec<f32> {
    let num_vectors = vectors.len();
    let num_dimensions = 250;

    let mut mean_vector = vec![0.0; num_dimensions];

    for vector in vectors {
        for i in 0..num_dimensions {
            mean_vector[i] += vector[i];
        }
    }

    for i in 0..num_dimensions {
        mean_vector[i] /= num_vectors as f32;
    }

    mean_vector
}

#[derive(Debug)]
struct FloatOrd(f32);

impl PartialEq for FloatOrd {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FloatOrd {}

impl PartialOrd for FloatOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for FloatOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverser l'ordre ici si nécessaire pour max-heap ou min-heap
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

fn filter_relevant_vectors<'a>(
    database: &'a [Vec<f32>],
    query_vectors: &[Vec<f32>],
    k: usize,
) -> Vec<(&'a [f32], usize)> {
    let query_refs: Vec<&[f32]> = query_vectors.iter().map(|v| &v[..]).collect();
    let mean_query_vector = calculate_mean_vector(&query_refs);

    let mut heap: BinaryHeap<(FloatOrd, usize)> = BinaryHeap::with_capacity(k);

    for (index, vector) in database.iter().enumerate() {
        let dist = squared_euclidean_distance(&mean_query_vector, vector);

        let ord_dist = FloatOrd(dist);
        // println!("{},{}", index, dist);
        // println!("{},{:#?}", index, ord_dist);
        if heap.len() < k {
            // println!("{},{:#?}", index, ord_dist);
            heap.push((ord_dist, index));
        } else if let Some(&(FloatOrd(top_dist), _)) = heap.peek() {
            if dist < top_dist {
                heap.pop();
                // println!("{},{:#?}", index, ord_dist);
                heap.push((ord_dist, index));
            }
        }
    }

    // Extract results from the heap
    let result: Vec<(&'a [f32], usize)> = heap
        .into_iter()
        .map(|(_, index)| (&database[index][..], index))
        .collect();

    result
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let start_total = Instant::now();

    let query_count = challenge.query_vectors.len();

    let subset_size = match query_count {
        10..=19 if challenge.difficulty.better_than_baseline <= 470 => 4200,
        10..=19 if challenge.difficulty.better_than_baseline > 470 => 4200,
        20..=28 if challenge.difficulty.better_than_baseline <= 465 => 3000,
        20..=28 if challenge.difficulty.better_than_baseline > 465 => 6000, // need more fuel
        29..=50 if challenge.difficulty.better_than_baseline <= 480 => 2000,
        29..=45 if challenge.difficulty.better_than_baseline > 480 => 6000,
        46..=50 if challenge.difficulty.better_than_baseline > 480 => 5000, // need more fuel
        51..=70 if challenge.difficulty.better_than_baseline <= 480 => 3000,
        51..=70 if challenge.difficulty.better_than_baseline > 480 => 3000, // need more fuel
        71..=100 if challenge.difficulty.better_than_baseline <= 480 => 1500,
        71..=100 if challenge.difficulty.better_than_baseline > 480 => 2500, // need more fuel
        _ => 10,                                                             // need more fuel
    };

    let start_filter = Instant::now();
    let subset = filter_relevant_vectors(
        &challenge.vector_database,
        &challenge.query_vectors,
        subset_size,
    );
    let duration_filter = start_filter.elapsed();
    println!(
        "Time taken for filtering relevant vectors: {:?}",
        duration_filter
    );

    let start_build_kd_tree = Instant::now();
    let kd_tree = build_kd_tree(&mut subset.clone());
    let duration_build_kd_tree = start_build_kd_tree.elapsed();
    println!(
        "Time taken for building KD-Tree: {:?}",
        duration_build_kd_tree
    );

    let start_search = Instant::now();
    let mut best_indexes = Vec::with_capacity(challenge.query_vectors.len());

    for query in challenge.query_vectors.iter() {
        let mut best = (std::f32::MAX, None);
        nearest_neighbor_search(&kd_tree, query, &mut best);

        if let Some(best_index) = best.1 {
            best_indexes.push(best_index);
        }
    }
    let duration_search = start_search.elapsed();
    println!(
        "Time taken for nearest neighbor search: {:?}",
        duration_search
    );

    let total_duration = start_total.elapsed();
    println!("Total time taken by solve_challenge: {:?}", total_duration);

    Ok(Some(Solution {
        indexes: best_indexes,
    }))
}

#[cfg(feature = "cuda")]
mod gpu_optimisation {
    use super::*;
    use cudarc::driver::*;
    use std::{collections::HashMap, sync::Arc};
    use tig_challenges::CudaKernel;

    // Set KERNEL to None if algorithm only has a CPU implementation
    pub const KERNEL: Option<CudaKernel> = Some(CudaKernel {
        src: r#"
        
        extern "C" __global__ void filter_vectors(float* query_mean, float* vectors, float* distances, int num_vectors, int num_dimensions) {
            int idx = blockIdx.x * blockDim.x + threadIdx.x;
            if (idx < num_vectors) {
                float dist = 0.0;
                for (int d = 0; d < num_dimensions; ++d) {
                    float diff = query_mean[d] - vectors[idx * num_dimensions + d];
                    dist += diff * diff;
                }
                distances[idx] = dist;
            }
        }
        
        "#,

        funcs: &["filter_vectors"],
    });

    fn compare_filter_results(
        subset_cpu: &Vec<(&[f32], usize)>,
        subset_gpu: &Vec<(&[f32], usize)>,
    ) -> bool {
        if subset_cpu.len() != subset_gpu.len() {
            println!(
                "Length mismatch: CPU subset length = {}, GPU subset length = {}",
                subset_cpu.len(),
                subset_gpu.len()
            );
            return false;
        }

        for i in 0..subset_cpu.len() {
            let (vector_cpu, index_cpu) = subset_cpu[i];
            let (vector_gpu, index_gpu) = subset_gpu[i];

            if index_cpu != index_gpu {
                println!(
                    "Index mismatch at position {}: CPU index = {}, GPU index = {}",
                    i, index_cpu, index_gpu
                );
                return false;
            }

            for j in 0..vector_cpu.len() {
                if (vector_cpu[j] - vector_gpu[j]).abs() > 1e-5 {
                    println!(
                        "Value mismatch at position [{}][{}]: CPU value = {}, GPU value = {}, difference = {}",
                        i,
                        j,
                        vector_cpu[j],
                        vector_gpu[j],
                        (vector_cpu[j] - vector_gpu[j]).abs()
                    );
                    return false;
                }
            }
        }
        true
    }

    use std::time::Instant;

    fn compare_distances_cpu_gpu(
        database: &[Vec<f32>],
        mean_query_vector: &[f32],
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<()> {
        let num_vectors = database.len();
        let num_dimensions = mean_query_vector.len();

        // Calcul des distances sur le CPU
        let start_cpu = Instant::now();
        let distances_cpu: Vec<f32> = database
            .iter()
            .map(|vector| squared_euclidean_distance(mean_query_vector, vector))
            .collect();
        let duration_cpu = start_cpu.elapsed();
        println!(
            "Time taken for calculating distances on CPU: {:?}",
            duration_cpu
        );

        // Calcul des distances sur le GPU
        let start_gpu = Instant::now();
        let flattened_database: Vec<f32> = database.iter().flatten().cloned().collect();
        let database_dev = dev.htod_sync_copy(&flattened_database)?;
        let mean_query_dev = dev.htod_sync_copy(&mean_query_vector)?;
        let mut distances_dev = dev.alloc_zeros::<f32>(num_vectors)?;

        let cfg = LaunchConfig {
            block_dim: (256, 1, 1),
            grid_dim: ((num_vectors as u32 + 255) / 256, 1, 1),
            shared_mem_bytes: 0,
        };

       
        unsafe {
            funcs.remove("filter_vectors").unwrap().launch(
                cfg,
                (
                    &mean_query_dev,
                    &database_dev,
                    &mut distances_dev,
                    num_vectors as i32,
                    num_dimensions as i32,
                ),
            )
        }?;
       
        let mut distances_gpu = vec![0.0f32; num_vectors];
        dev.dtoh_sync_copy_into(&distances_dev, &mut distances_gpu)?;
        let duration_gpu = start_gpu.elapsed();
        println!(
            "Time taken for calculating distances on GPU: {:?}",
            duration_gpu
        );

        // Comparaison des distances CPU et GPU
        let mut discrepancies = 0;
        for (i, (dist_cpu, dist_gpu)) in distances_cpu.iter().zip(&distances_gpu).enumerate() {
            if (dist_cpu - dist_gpu).abs() > 1e-5 {
                println!(
                "Discrepancy found at index {}: CPU distance = {}, GPU distance = {}, difference = {}",
                i,
                dist_cpu,
                dist_gpu,
                (dist_cpu - dist_gpu).abs()
            );
                discrepancies += 1;
            }
        }

        if discrepancies == 0 {
            println!("No discrepancies found between CPU and GPU distances.");
        } else {
            println!("Total discrepancies found: {}", discrepancies);
        }

        Ok(())
    }

    pub fn cuda_solve_challenge(
        challenge: &Challenge,
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<Option<Solution>> {
        let query_count = challenge.query_vectors.len();

        let subset_size = match query_count {
            10..=19 if challenge.difficulty.better_than_baseline <= 470 => 4200,
            10..=19 if challenge.difficulty.better_than_baseline > 470 => 4200,
            20..=28 if challenge.difficulty.better_than_baseline <= 465 => 3000,
            20..=28 if challenge.difficulty.better_than_baseline > 465 => 6000,
            29..=50 if challenge.difficulty.better_than_baseline <= 480 => 2000,
            29..=45 if challenge.difficulty.better_than_baseline > 480 => 6000,
            46..=50 if challenge.difficulty.better_than_baseline > 480 => 5000,
            51..=70 if challenge.difficulty.better_than_baseline <= 480 => 3000,
            51..=70 if challenge.difficulty.better_than_baseline > 480 => 3000,
            71..=100 if challenge.difficulty.better_than_baseline <= 480 => 1500,
            71..=100 if challenge.difficulty.better_than_baseline > 480 => 2500,
            _ => 10,
        };

       

        // Filtrage des vecteurs pertinents sur CPU
        let start_filter = Instant::now();
        let subset_cpu = filter_relevant_vectors(
            &challenge.vector_database,
            &challenge.query_vectors,
            subset_size,
        );
        let duration_filter_cpu = start_filter.elapsed();
        println!(
            "Time taken for filtering relevant vectors on CPU: {:?}",
            duration_filter_cpu
        );

        // Filtrage des vecteurs pertinents sur GPU
        let start_filter_gpu = Instant::now();
        let subset_gpu = cuda_filter_relevant_vectors(
            &challenge.vector_database,
            &challenge.query_vectors,
            subset_size,
            dev,
            funcs,
        )?;
        let duration_filter_gpu = start_filter_gpu.elapsed();
        println!(
            "Time taken for filtering relevant vectors on GPU: {:?}",
            duration_filter_gpu
        );

        // Comparaison des résultats
        let results_match = compare_filter_results(&subset_cpu, &subset_gpu);

        if results_match {
            println!("CPU and GPU results match.");
        } else {
            println!("CPU and GPU results do NOT match!");
        }

        let start_build_kd_tree = Instant::now();
        let kd_tree = build_kd_tree(&mut subset_gpu.clone());
        let duration_build_kd_tree = start_build_kd_tree.elapsed();
        println!(
            "Time taken for building KD-Tree: {:?}",
            duration_build_kd_tree
        );

        //let start_search = Instant::now();
        let mut best_indexes = Vec::with_capacity(challenge.query_vectors.len());

        for query in challenge.query_vectors.iter() {
            let mut best = (std::f32::MAX, None);
            nearest_neighbor_search(&kd_tree, query, &mut best);

            if let Some(best_index) = best.1 {
                best_indexes.push(best_index);
            }
        }
        //let duration_search = start_search.elapsed();
        //println!("Time taken for nearest neighbor search: {:?}", duration_search);

        //let kd_tree = cuda_build_kd_tree(&mut subset.clone(), dev, funcs);

        // let mut best_indexes = Vec::with_capacity(challenge.query_vectors.len());
        // for query in challenge.query_vectors.iter() {
        //     let mut best = (std::f32::MAX, None);
        //     cuda_nearest_neighbor_search(&kd_tree, query, &mut best, dev, funcs)?;

        //     if let Some(best_index) = best.1 {
        //         best_indexes.push(best_index);
        //     }
        // }

        Ok(Some(Solution {
            indexes: best_indexes,
        }))
    }

    #[cfg(feature = "cuda")]
    fn cuda_filter_relevant_vectors<'a>(
        database: &'a [Vec<f32>],
        query_vectors: &[Vec<f32>],
        k: usize,
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<Vec<(&'a [f32], usize)>> {
        let query_refs: Vec<&[f32]> = query_vectors.iter().map(|v| &v[..]).collect();
        let mean_query_vector = calculate_mean_vector(&query_refs);

        // compare_distances_cpu_gpu(
        //     &database,
        //     &mean_query_vector,
        //     dev,
        //     funcs.clone(),
        // )?;

        let num_vectors = database.len();
        let num_dimensions = 250;

        let flattened_database: Vec<f32> = database.iter().flatten().cloned().collect();

        let database_dev = dev.htod_sync_copy(&flattened_database)?;
        let mean_query_dev = dev.htod_sync_copy(&mean_query_vector)?;
        let mut distances_dev = dev.alloc_zeros::<f32>(num_vectors)?;

        let cfg = LaunchConfig {
            block_dim: (256, 1, 1),
            grid_dim: ((num_vectors as u32 + 255) / 256, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            funcs.remove("filter_vectors").unwrap().launch(
                cfg,
                (
                    &mean_query_dev,
                    &database_dev,
                    &mut distances_dev,
                    num_vectors as i32,
                    num_dimensions as i32,
                ),
            )
        }?;

        let mut distances_host = vec![0.0f32; num_vectors];
        dev.dtoh_sync_copy_into(&distances_dev, &mut distances_host)?;

        let mut heap: BinaryHeap<(FloatOrd, usize)> = BinaryHeap::with_capacity(k);
        // println!("compare to GPU");
        for (index, &distance) in distances_host.iter().enumerate() {
            let ord_dist = FloatOrd(distance);
            // println!("{},{}", index, dist);
            // println!("{},{:#?}", index, ord_dist);
    
            if heap.len() < k {
                heap.push((ord_dist, index));
            } else if let Some(&(FloatOrd(top_dist), _)) = heap.peek() {
                if distance < top_dist {
                    heap.pop();
                    heap.push((ord_dist, index));
                }
            }
        }

        let result: Vec<(&'a [f32], usize)> = heap
            .into_iter()
            .map(|(_, index)| (&database[index][..], index))
            .collect();

        Ok(result)
    }

    #[cfg(feature = "cuda")]
    fn cuda_build_kd_tree<'a>(
        subset: &mut [(&'a [f32], usize)],
        dev: &Arc<CudaDevice>,
        funcs: &mut HashMap<&'static str, CudaFunction>,
    ) -> Option<Box<KDNode<'a>>> {
        None
    }

    #[cfg(feature = "cuda")]
    fn cuda_nearest_neighbor_search(
        kd_tree: &Option<Box<KDNode<'_>>>,
        query: &[f32],
        best: &mut (f32, Option<usize>),
        dev: &Arc<CudaDevice>,
        funcs: &mut HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
#[cfg(feature = "cuda")]
pub use gpu_optimisation::{cuda_solve_challenge, KERNEL};
