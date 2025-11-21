pub mod filter;
pub mod resampler;

use crate::apu;
use crate::apu::APU;
use crate::dac::filter::Filter;
use crate::dac::resampler::Resampler;


pub struct DAC {
    sample_rate: u32,
    clock_skip: u32,
    resampler: Resampler,
    filters: Vec<Filter>,
    sample_count: u32,
}

impl DAC {
    pub fn new(sampling_rate: u32, clock_skip: u32) -> DAC {
        DAC {
            sample_rate: sampling_rate,
            clock_skip: clock_skip,
            resampler: Resampler::new(apu::MAIN_FREQ/(clock_skip+1), sampling_rate),
            filters: vec![
                Filter::hipass_1p_iir(90, sampling_rate),
                Filter::hipass_1p_iir(440, sampling_rate),
                Filter::lowpass_1p_iir(14000, sampling_rate),
            ], //High pass 90hz, high pass 440Hz, low pass 14kHz. All first order
            sample_count: 0,
        }
    }

    pub fn advance_sample(&mut self, apu: &mut APU) -> f32 {
        let mut sample: f32;

        loop {
            for _ in 0..(self.clock_skip+1) {
                apu.clock();
            }
            if let Some(sp) = self.resampler.tick(apu.output()) {
                sample = sp;
                break;
            }
        }

        self.sample_count += 1;
        for filter in self.filters.iter_mut() {
            sample = filter.step(sample)
        }
        return sample
    }

    pub fn advance_miliseconds(&mut self, apu: &mut APU, time: f32) -> Vec<f32> {
        //let mut samples: Vec<f32> = Vec::new()

        //TODO handle when division isnt exact. Maybe change to ceil and save extra elapsed time in struct 
        let num_samples = ((self.sample_rate as f32) * time/1000.).round() as u32;

        (0..num_samples).into_iter().map(
            |_| {self.advance_sample(apu)}
        ).collect()

    }
}