use super::types::*;

pub fn compute_k(qlen: usize, diagonal: Diagonal) -> usize {
    ((qlen as isize - 1) + diagonal as isize) as usize
}

pub fn compute_v(offset: Offset, diagonal: Diagonal, central_diagonal: usize) -> usize {
    let central_diagonal = central_diagonal as i32;
    let offset = offset as usize;

    if diagonal <= central_diagonal {
        offset
    } else {
        offset + abs!(diagonal - central_diagonal) as usize
    }
}

pub fn compute_h(offset: Offset, diagonal: Diagonal, central_diagonal: usize) -> usize {
    let central_diagonal = central_diagonal as i32;
    let offset = offset as usize;

    if diagonal <= central_diagonal {
        offset + abs!(diagonal - central_diagonal) as usize
    } else {
        offset
    }
}
