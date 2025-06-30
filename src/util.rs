
use hound;


pub fn encode_dm(signal: &Vec<f32>, max_val: f32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut encoded: Vec<f32> = Vec::new();

    
    let delta: f32 = 2.*max_val/128.;
    let lim_pos = 63.*delta;
    let lim_neg = -64.*delta;

    let mut curr_value: f32 = 0.;
    let mut bit_count: u8 = 0;
    let mut code: u8 = 0;
    for &sample in signal {
        let bit: u8 = if curr_value < sample {
            curr_value += 2.*delta;
            curr_value = curr_value.min(lim_pos);
            1
        } else {
            curr_value -= 2.*delta;
            curr_value = curr_value.max(lim_neg);
            0
        };
        code |= bit << bit_count;
        bit_count += 1;
        if bit_count == 8 {
            bit_count = 0;
            out.push(code);
            code = 0;
        }
        #[cfg(debug_assertions)]
        encoded.push(curr_value);
    }
    while bit_count < 8 {
        let bit: u8 = if curr_value >= 0. { 1} else {0};
        code |= bit << bit_count;
        bit_count += 1;
    }
    out.push(code);

    #[cfg(debug_assertions)]
    {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 33144,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create("dm_encoded.wav", spec).unwrap();
        for sample in encoded {
            writer.write_sample(sample).unwrap();
        }
    }

    out
}