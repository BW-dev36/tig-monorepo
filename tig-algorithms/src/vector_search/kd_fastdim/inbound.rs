use anyhow::Ok;
use tig_challenges::vector_search::*;

struct KDNode {
    point: Vec<f32>,
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
    index: usize,
}

impl KDNode {
    fn new(point: Vec<f32>, index: usize) -> Self {
        KDNode {
            point,
            left: None,
            right: None,
            index,
        }
    }
}

fn build_kd_tree(points: &mut [(Vec<f32>, usize)], selected_dims: &[usize]) -> Option<Box<KDNode>> {
    if points.is_empty() {
        return None;
    }

    let mut stack: Vec<(usize, usize, usize, Option<*mut KDNode>, bool)> = Vec::new();
    let mut root: Option<Box<KDNode>> = None;

    stack.push((0, points.len(), 0, None, false));

    while let Some((start, end, depth, parent_ptr, is_left)) = stack.pop() {
        if start >= end {
            continue;
        }

        let axis = selected_dims[depth % selected_dims.len()];
        let median = (start + end) / 2;

        points[start..end].sort_unstable_by(|a, b| a.0[axis].partial_cmp(&b.0[axis]).unwrap());

        let (median_point, median_index) = points[median].clone();
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
    a.iter()
        .zip(b)
        .map(|(&x1, &x2)| (x1 - x2) * (x1 - x2))
        .sum()
}

fn nearest_neighbor_search(
    root: &Option<Box<KDNode>>,
    target: &[f32],
    depth: usize,
    best: &mut (f32, Option<usize>),
    selected_dims: &[usize],
) {
    if let Some(node) = root {
        let axis = selected_dims[depth % selected_dims.len()];
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

        nearest_neighbor_search(nearer, target, depth + 1, best, selected_dims);

        if diff * diff < best.0 {
            nearest_neighbor_search(farther, target, depth + 1, best, selected_dims);
        }
    }
}

fn calculate_mean_vector(vectors: &[Vec<f32>]) -> Vec<f32> {
    let num_vectors = vectors.len();
    let num_dimensions = vectors[0].len();

    let mut mean_vector = vec![0.0; num_dimensions];

    for vector in vectors {
        for (i, &value) in vector.iter().enumerate() {
            mean_vector[i] += value;
        }
    }

    for value in &mut mean_vector {
        *value /= num_vectors as f32;
    }

    mean_vector
}

fn filter_relevant_vectors<'a>(
    database: &'a [Vec<f32>],
    query_vectors: &[Vec<f32>],
    k: usize,
) -> Vec<(Vec<f32>, usize)> {
    let mean_query_vector = calculate_mean_vector(query_vectors);

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
        .map(|(index, _)| (database[index].clone(), index))
        .collect()
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {

    //10, 460 => subet_size 4200
    //20, 460 => subet_size 1300
    //30, 460 => subet_size 1300
    //40, 460 => subet_size 1300
    //60, 460 => subet_size 1300
    //70, 460 => subet_size 1300

    let query_count = challenge.query_vectors.len();

    // Determine subset_size
    let subset_size = match query_count {
        10..=19 if challenge.difficulty.better_than_baseline <= 490 => 4200,
        20..=70 if challenge.difficulty.better_than_baseline <= 460 => 1300,
        _ => 5000, // Valeur par défaut si aucune correspondance n'est trouvée
    };

    let subset = filter_relevant_vectors(
        &challenge.vector_database,
        &challenge.query_vectors,
        subset_size,
    );

    //TODO Better performance later ?
    let selected_dims: Vec<usize> = (0..250).collect();

    let kd_tree = build_kd_tree(&mut subset.clone(), &selected_dims);

    let mut best_indexes = Vec::with_capacity(challenge.query_vectors.len());

    for query in challenge.query_vectors.iter() {
        let mut best = (std::f32::MAX, None);
        nearest_neighbor_search(&kd_tree, query, 0, &mut best, &selected_dims);

        if let Some(best_index) = best.1 {
            best_indexes.push(best_index);
        }
    }

    Ok(Some(Solution {
        indexes: best_indexes,
    }))
}
