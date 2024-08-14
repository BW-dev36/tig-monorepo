use anyhow::Ok;
use tig_challenges::vector_search::*;

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

fn build_kd_tree<'a>(
    points: &mut [(&'a [f32], usize)],
) -> Option<Box<KDNode<'a>>> {
    if points.is_empty() {
        return None;
    }

    let num_dimensions = points[0].0.len(); // Nombre total de dimensions des vecteurs
    let mut stack: Vec<(usize, usize, usize, Option<*mut KDNode<'a>>, bool)> = Vec::new();
    let mut root: Option<Box<KDNode<'a>>> = None;

    stack.push((0, points.len(), 0, None, false));

    let mut axis = 0;

    while let Some((start, end, depth, parent_ptr, is_left)) = stack.pop() {
        if start >= end {
            continue;
        }

        let median = (start + end) >> 1;

        points[start..end].sort_unstable_by(|a, b| a.0[axis].partial_cmp(&b.0[axis]).unwrap());

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

        axis += 1;
        if axis >= num_dimensions {
            axis = 0;
        }
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
        let dist = squared_euclidean_distance(&node.point, target);

        if dist < best.0 {
            best.0 = dist;
            best.1 = Some(node.index);
        }

        let diff = target[axis] - node.point[axis];
        let (nearer, farther) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(farther_node) = farther {
            if diff * diff < best.0 {
                stack.push((farther_node.as_ref(), depth + 1));
            }
        }

        if let Some(nearer_node) = nearer {
            stack.push((nearer_node.as_ref(), depth + 1));
        }
    }
}

fn calculate_mean_vector(vectors: &[&[f32]]) -> Vec<f32> {
    let num_vectors = vectors.len();
    let num_dimensions = vectors[0].len();

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

fn filter_relevant_vectors<'a>(
    database: &'a [Vec<f32>],
    query_vectors: &[Vec<f32>],
    k: usize,
) -> Vec<(&'a [f32], usize)> {
    let query_refs: Vec<&[f32]> = query_vectors.iter().map(|v| &v[..]).collect();
    let mean_query_vector = calculate_mean_vector(&query_refs);

    let mut distances: Vec<(usize, f32)> = database
        .iter()
        .enumerate()
        .map(|(index, vector)| {
            let dist = squared_euclidean_distance(&mean_query_vector, vector);
            (index, dist)
        })
        .collect();

    distances.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    distances
        .into_iter()
        .take(k)
        .map(|(index, _)| (&database[index][..], index))
        .collect()
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let query_count = challenge.query_vectors.len();

    let subset_size = match query_count {
        10..=19 if challenge.difficulty.better_than_baseline <= 470 => 4200,
        10..=19 if challenge.difficulty.better_than_baseline > 470 => 5000,
        20..=28 if challenge.difficulty.better_than_baseline <= 465 => 3000,
        20..=28 if challenge.difficulty.better_than_baseline > 465 => 10000, // need more fuel
        29..=50 if challenge.difficulty.better_than_baseline <= 480 => 2000,
        29..=50 if challenge.difficulty.better_than_baseline > 480 => 10000, // need more fuel
        51..=70 if challenge.difficulty.better_than_baseline <= 480 => 1300,
        51..=70 if challenge.difficulty.better_than_baseline > 480 => 10000, // need more fuel
        71..=100 if challenge.difficulty.better_than_baseline <= 445 => 1000,
        71..=100 if challenge.difficulty.better_than_baseline > 445 => 10000, // need more fuel
        _ => 15000, // need more fuel
    };

    let subset = filter_relevant_vectors(
        &challenge.vector_database,
        &challenge.query_vectors,
        subset_size,
    );

    let kd_tree = build_kd_tree(&mut subset.clone());

    let mut best_indexes = Vec::with_capacity(challenge.query_vectors.len());

    for query in challenge.query_vectors.iter() {
        let mut best = (std::f32::MAX, None);
        nearest_neighbor_search(&kd_tree, query, &mut best);

        if let Some(best_index) = best.1 {
            best_indexes.push(best_index);
        }
    }

    Ok(Some(Solution {
        indexes: best_indexes,
    }))
}
