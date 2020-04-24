#[macro_use]
extern crate vst;
extern crate rand;


use vst::plugin::{Info, Plugin, Category};
use vst::buffer::AudioBuffer;
use rand::random;
use vst::api::Events;
use vst::event::Event;
use std::f64::consts::PI;

const TAU: f64 = PI * 2.0;


struct Whisper {
    notes: u8,
    note: u8,
    sample_rate: f32,
    time: f64,
    note_duration: f64,
    decay_time: f64
}

impl Default for Whisper {
    fn default() -> Whisper {
        Whisper {
            notes: 0,
            note: 0,
            sample_rate: 44100.0,
            time: 0.0,
            note_duration: 0.0,
            decay_time: 0.0
        }
    }
}


impl Plugin for Whisper {
    fn get_info(&self) -> Info {
        Info {
            name: "Whisper".to_string(),
            unique_id: 1337,

            inputs: 0,
            outputs: 2,

            category: Category::Synth,

            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        144 => {
                            self.notes += 1;
                            self.note_duration = 0.0;

                        },
                        128 => {
                            self.notes -= 1;
                            self.decay_time = 1.0
                        },
                        _ => ()
                    }

                    self.note = ev.data[1];
                }

                _ => ()
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }



    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = (1.0 / self.sample_rate) as f64;

        let mut output_sample;
        for sample_idx in 0..samples {
            let time = self.time;

            if self.notes != 0 || self.decay_time > 0.0 {
                let signal = (time * midi_pitch_to_freq(self.note) * TAU).sin();

                let attack = 0.5;
                let alpha = if self.note_duration < attack {
                    self.note_duration / attack
                } else if self.notes == 0 {
                    self.decay_time
                } else {
                    1.0
                };

                output_sample = (signal * alpha) as f32;

                self.time += per_sample;
                self.note_duration += per_sample;
            } else {
                output_sample = 0.0;
            }

            if self.notes == 0 && self.decay_time > 0.0 {
                self.decay_time -= per_sample;
            }

            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }
}


plugin_main!(Whisper);



fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
}
