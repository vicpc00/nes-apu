use std::collections::VecDeque;
use crate::dac::filter::Filter;

pub struct Resampler {
    up_ratio: u32,
    down_ratio: u32,
    filter: Vec<f32>,
    input_buffer: VecDeque<f32>,
    output_buffer: VecDeque<f32>,
}

impl Resampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Resampler {
        let ratio = (output_rate as f32)/(input_rate as f32);
        let (n, d) = rational_fraction_approximation(ratio, 0.0001);
        let filt = Filter::lowpass_blackman(2*10*n.max(d), (output_rate/2) as f32, input_rate as f32);
        Resampler {
            up_ratio: n,
            down_ratio: d,
            filter: filt.direct_coeff,
            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new(),
        }
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