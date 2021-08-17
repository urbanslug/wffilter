pub type Offset = u64;
pub type Diagonal = i32;
pub type Score = usize;


pub struct WaveFront {
    pub high: Diagonal,
    pub low: Diagonal,
    pub m_wavefront: Vec<Offset>,
    pub i_wavefront: Vec<Offset>,
    pub d_wavefront: Vec<Offset>,
}

impl WaveFront {
    pub fn new(high: Diagonal, low: Diagonal, size: usize) -> Self {
        Self {
            high,
            low,
            m_wavefront: vec![0; size],
            i_wavefront: vec![0; size],
            d_wavefront: vec![0; size],
        }
    }
}

pub struct WaveFronts {
    pub wavefronts: Vec<WaveFront>,
    max_offset: usize,
}

impl WaveFronts {
    pub fn new(tlen: usize, qlen: usize) -> Self {
        Self {
            wavefronts: Vec::with_capacity(tlen+qlen),
            max_offset: tlen+qlen,
        }
    }

    pub fn add_wavefront(&mut self, score: Score, hi: Diagonal, lo: Diagonal) {
        let score = score as usize;
        let num_wavefronts = self.wavefronts.len();
        let size = self.max_offset;

        for _ in num_wavefronts..=score {
            self.wavefronts.push(WaveFront::new(hi, lo, size));
        }
    }
}
