use std::collections::VecDeque;
use crate::dac::filter::Filter;

pub struct Resampler {
    up_ratio: usize,
    down_ratio: usize,
    filter: Vec<f32>,
    input_buffer: VecDeque<f32>,
    sample_counter: usize,
}

impl Resampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Resampler {
        let ratio = (output_rate as f32)/(input_rate as f32);
        let (n, d) = rational_fraction_approximation(ratio, 0.0001);

        #[cfg(debug_assertions)]
        println!("Ratios: n={}, d={}. Actual output rate: {}. Tuning error: {:+.1}c", 
            n, d, input_rate*n/d, 
            1200. * f32::log2((output_rate as f32) / ((input_rate*n/d) as f32) )
        );
        let filt_len = 2*10*u32::max(n, d);
        let mut filt = Filter::lowpass_blackman_norm((filt_len/2) as usize,
            0.95/(u32::max(n, d) as f32)
        ).direct_coeff;

        let g = n as f32;
        for coeff in &mut filt {
            *coeff *= g
        }
        let input_buffer_size = ((filt_len + n)/n) as usize; //ceil of filt.len()+1 and n
        //TODO: otimização -> precomputar os filtros decimados. seria uma matrix n x ceil(l_f / n)
        // Talves representar como um vetor 1D com slices sendo cada filtro, a la numpy
        Resampler {
            up_ratio: n as usize,
            down_ratio: d as usize,
            filter: filt,
            input_buffer: VecDeque::from(vec![0.; input_buffer_size]),
            sample_counter: 0,
        }
    }

    pub fn tick(&mut self, sample:f32) -> Option<f32> {
        self.input_buffer.push_front(sample);
        self.input_buffer.pop_back();
        self.sample_counter += self.up_ratio;
        if self.sample_counter < self.down_ratio {
            return None
        }
        self.sample_counter -= self.down_ratio;

        let mut out = 0.0;
        let mut idx_filt = self.sample_counter;
        let mut idx_in = 0;
        while idx_filt < self.filter.len() {
            //Filter is assumed to be symetric, so take idx instead of len - idx - 1 
            out += self.input_buffer[idx_in] * self.filter[idx_filt];  
            idx_filt += self.up_ratio;
            idx_in += 1;
        }
        // TODO: maybe compensate delay. 
        Some(out)
    }
}

#[allow(non_snake_case)]
pub fn rational_fraction_approximation(ratio: f32, tolerance: f32) -> (u32, u32) {

    let mut x = 1./ratio.fract();

    let mut A_curr = ratio.trunc();
    let mut B_curr = 1.;
    let mut A_prev = 1.;
    let mut B_prev = 0.;

    while (ratio - A_curr/B_curr).abs() > tolerance * ratio {
        let bn = x.trunc();
        x = 1./x.fract();
        let A_next = bn*A_curr + A_prev;
        let B_next = bn*B_curr + B_prev;

        A_prev = A_curr;
        A_curr = A_next;
        B_prev = B_curr;
        B_curr = B_next;
    }
    (A_curr as u32, B_curr as u32)
}