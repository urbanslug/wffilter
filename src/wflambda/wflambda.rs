use super::types::*;
use super::utils;

fn wf_extend<F>(
    wavefront: &mut WaveFront,
    match_lambda: &F,
    _tlen: usize,
    qlen: usize
) where
    F: Fn(usize, usize) -> bool
{
    let a_k = 0; // FIXME: make me an arg or something

    let m_wavefront: &mut Vec<Offset> = &mut wavefront.m_wavefront;

    for k in wavefront.low..=wavefront.high {
        let k_idx = utils::compute_k(qlen, k);
        let offset = m_wavefront[k_idx];

        let v = utils::compute_v(offset, k, a_k);
        let h = utils::compute_h(offset, k, a_k);

        eprintln!("v={}, h={} offset={}, k={}, k_idx={}", v, h, offset, k, k_idx);

        while match_lambda(v, h) {
            m_wavefront[k_idx] += 1; // FIXME: should this be increased by the number of matches?
        }
    }
}

#[allow(unused_mut, unused_variables)]
pub fn wf_align<F, G>(
    match_lambda: &F,
    traceback_lambda: &G,
    tlen: usize,
    qlen: usize,
)
where
    F: Fn(usize, usize) -> bool,
    G: Fn((i32, i32), (i32, i32)),
{
    // init wavefronts
    let mut wavefronts = WaveFronts::new(tlen, qlen);
    let mut score: Score = 0;

    let a_k = tlen - qlen; // central_diagonal
    let a_k = a_k as i32;

    wavefronts.add_wavefront(score, a_k, a_k);

    // eprintln!("m={} n={}", tlen,qlen);

    loop {
        let mut wf = match wavefronts.wavefronts.get_mut(score) {
            Some(wf) => wf,
            _ => panic!("wf_align couldn't get wavefront at score"),
        };

        wf_extend(&mut wf, match_lambda, tlen, qlen);

        break;
        //score += 1;
    }


}
