
use hound;

use nes_apu::{apu::APU, dac::filter::Filter, util};
use nes_apu::dac::DAC;

fn main() {
    ptn_test();
    //dmc_test();
}

//TODO: Organize this stuff as actual unity tests
fn ptn_test() {
    let mut apu = APU::new();

    apu.write_opp(0x4015, 0b0000_0001);
    apu.write_opp(0x4017, 0b0000_0000);

    //apu.write_opp(0x4000, 0b1000_0011);
    apu.write_opp(0x4000, 0b1001_1111);
    //apu.write_opp(0x4001, 0b1000_1001);
    apu.write_opp(0x4001, 0b0000_1000);
    apu.write_opp(0x4002, 0xFD); //440Hz = 0x0FD
    apu.write_opp(0x4003, (0x16 << 3) + 0x00);

    apu.write_opp(0x4008, 0b0111_1111);
    apu.write_opp(0x400A, 0x7E);
    apu.write_opp(0x400B, (0x14 << 3) + 0x00);

    apu.write_opp(0x400C, 0b0000_0011);
    apu.write_opp(0x400E, 0x07);
    apu.write_opp(0x400F, 0x16 << 3);


    println!("{:?}", apu.pulse1);
    println!("{:?}", apu.pulse2);
    println!("{:?}", apu.triangle);
    println!("{:?}", apu.noise);

    let final_fs = 44100;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: final_fs,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

    let mut dac = DAC::new(final_fs, 0);

    let samples = dac.advance_miliseconds(&mut apu, 1000.);
    for sample in samples.into_iter() {
        writer.write_sample(sample).unwrap();
    }
}

fn dmc_test() {
    let mut apu = APU::new();

    let mut reader = hound::WavReader::open("pcm-sample.wav").unwrap();
    let signal: Vec<f32> = reader.samples::<f32>()
                                 .map(|s| {s.unwrap()})
                                 .collect();
    let dm_encoded = util::encode_dm(&signal, 0.5);

    apu.load_sample(0xC000, &dm_encoded);

    apu.write_opp(0x4010, 0b0100_1111);
    apu.write_opp(0x4011, 0x40); //64, half of range
    apu.write_opp(0x4012, 0x00);
    apu.write_opp(0x4013, 0xFF);

    apu.write_opp(0x4015, 0b0001_0000);

    println!("{:?}", apu.dmc);

    let final_fs = 44100;
    let apu_downsample = 2;
    let apu_fs = apu_downsample*final_fs;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: final_fs,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let cps = (21_477_270/6)/apu_fs;

    let mut aa_filter = Filter::lowpass_blackman(128, 20000, apu_fs);

    let mut writer = hound::WavWriter::create("out_dmc.wav", spec).unwrap();

    for sample_count in 0 .. 3*44100 {
        for _ in 0..cps {
            apu.clock();
        }
        let sample = apu.output();
        let filtered_sample = aa_filter.step(sample);
        if sample_count % apu_downsample == 0 {
            writer.write_sample(filtered_sample).unwrap();
        }
    }
}