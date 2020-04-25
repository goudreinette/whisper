#[macro_use]
extern crate vst;
extern crate rand;
extern crate lerp;
extern crate noise;
extern crate vst_gui;


use vst::plugin::{Info, Plugin, Category, PluginParameters};
use vst::buffer::AudioBuffer;
use rand::random;
use vst::api::Events;
use vst::event::Event;
use std::f64::consts::PI;
use noise::{NoiseFn, Perlin, Worley, Point2, Billow, Cylinders, OpenSimplex, RidgedMulti, SuperSimplex, Value, HybridMulti, BasicMulti};
use vst::editor::Editor;
use std::sync::{Arc, Mutex, MutexGuard};
use vst_gui::{JavascriptCallback, PluginGui};
use vst::util::AtomicFloat;
use std::ops::Deref;
use std::rc::Rc;


const TAU: f64 = PI * 2.0;

const HTML: &'static str = include_str!("./ui.html");

static mut LAST_SAMPLE : f64 = 0.0;

#[derive(Debug, Copy, Clone)]
struct Note {
    alpha: f64,
    note: u8,
    is_released: bool,
}


struct Whisper {
    notes: Vec<Note>,

    sample_rate: f32,
    time: f64,

    // Noise functions
    fn_perlin: Perlin,
    fn_value: Value,
    fn_worley: Worley,
    fn_ridged_multi: RidgedMulti,
    fn_open_simplex: OpenSimplex,
    fn_billow: Billow,
    fn_cylinders: Cylinders,
    fn_hybrid_multi: HybridMulti,
    fn_basic_multi: BasicMulti,

    params: Arc<WhisperParameters>,

    last_sample: f32,
}


/*
|--------------------------------------------------------------------------
| Parameters
|--------------------------------------------------------------------------
*/
struct WhisperParameters {
    // Amounts
    a_white_noise: AtomicFloat,
    a_perlin: AtomicFloat,
    a_value: AtomicFloat,
    a_worley: AtomicFloat,
    a_ridged_multi: AtomicFloat,
    a_open_simplex: AtomicFloat,
    a_billow: AtomicFloat,
    a_cylinders: AtomicFloat,
    a_hybrid_multi: AtomicFloat,
    a_basic_multi: AtomicFloat,

    attack_duration: AtomicFloat,
    release_duration: AtomicFloat,
}

impl PluginParameters for WhisperParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.a_white_noise.get(),
            1 => self.a_perlin.get(),
            2 => self.a_value.get(),
            3 => self.a_worley.get(),
            4 => self.a_ridged_multi.get(),
            5 => self.a_open_simplex.get(),
            6 => self.a_billow.get(),
            7 => self.a_cylinders.get(),
            8 => self.a_hybrid_multi.get(),
            9 => self.a_basic_multi.get(),
            10 => self.attack_duration.get(),
            11 => self.release_duration.get(),
            _ => 0.0,
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.1}%", self.a_white_noise.get() * 100.0),
            1 => format!("{:.1}%", self.a_perlin.get() * 100.0),
            2 => format!("{:.1}%", self.a_value.get() * 100.0),
            3 => format!("{:.1}%", self.a_worley.get() * 100.0),
            4 => format!("{:.1}%", self.a_ridged_multi.get() * 100.0),
            5 => format!("{:.1}%", self.a_open_simplex.get() * 100.0),
            6 => format!("{:.1}%", self.a_billow.get() * 100.0),
            7 => format!("{:.1}%", self.a_cylinders.get() * 100.0),
            8 => format!("{:.1}%", self.a_hybrid_multi.get() * 100.0),
            9 => format!("{:.1}%", self.a_basic_multi.get() * 100.0),
            10 => format!("{:.1}s", self.attack_duration.get()),
            11 => format!("{:.1}s", self.release_duration.get()),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "White",
            1 => "Perlin",
            2 => "Value",
            3 => "Worley",
            4 => "RidgedMulti",
            5 => "OpenSimplex",
            6 => "Billow",
            7 => "Cylinders",
            8 => "HybridMulti",
            9 => "BasicMulti",
            10 => "Attack",
            11 => "Release",
            _ => "",
        }.to_string()
    }

    fn set_parameter(&self, index: i32, val: f32) {
        match index {
            0 => self.a_white_noise.set(val),
            1 => self.a_perlin.set(val),
            2 => self.a_value.set(val),
            3 => self.a_worley.set(val),
            4 => self.a_ridged_multi.set(val),
            5 => self.a_open_simplex.set(val),
            6 => self.a_billow.set(val),
            7 => self.a_cylinders.set(val),
            8 => self.a_hybrid_multi.set(val),
            9 => self.a_basic_multi.set(val),
            10 => self.attack_duration.set(val.max(0.001)), // prevent division by zero
            11 => self.release_duration.set(val.max(0.001)),
            _ => (),
        }
    }
}

impl Default for WhisperParameters {
    fn default() -> WhisperParameters {
        WhisperParameters {
            a_white_noise: AtomicFloat::new(1.0),
            a_perlin: AtomicFloat::new(0.0),
            a_value: AtomicFloat::new(0.0),
            a_worley: AtomicFloat::new(0.0),
            a_ridged_multi: AtomicFloat::new(0.0),
            a_open_simplex: AtomicFloat::new(0.0),
            a_billow: AtomicFloat::new(0.0),
            a_cylinders: AtomicFloat::new(0.0),
            a_hybrid_multi: AtomicFloat::new(0.0),
            a_basic_multi: AtomicFloat::new(0.0),
            attack_duration: AtomicFloat::new(0.5),
            release_duration: AtomicFloat::new(0.5),
        }
    }
}

impl Default for Whisper {
    fn default() -> Whisper {
        Whisper {
            notes: vec![],
            sample_rate: 44100.0,
            time: 0.0,

            // Amounts
            params: Arc::new(WhisperParameters::default()),

            // Noise functions
            fn_perlin: Perlin::new(),
            fn_value: Value::new(),
            fn_worley: Worley::new(),
            fn_ridged_multi: RidgedMulti::new(),
            fn_open_simplex: OpenSimplex::new(),
            fn_billow: Billow::new(),
            fn_cylinders: Cylinders::new(),
            fn_hybrid_multi: HybridMulti::new(),
            fn_basic_multi: BasicMulti::new(),

            last_sample: 0.0,
        }
    }
}


impl Plugin for Whisper {
    fn get_info(&self) -> Info {
        Info {
            name: "WhisperRRR".to_string(),
            unique_id: 1337,

            inputs: 0,
            outputs: 2,
            parameters: 12,

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
                            self.notes.push(Note { note: ev.data[1], alpha: 0.0, is_released: false });
                        }
                        128 => {
                            for note in self.notes.iter_mut() {
                                if note.note == ev.data[1] {
                                    note.is_released = true;
                                }
                            }
                        }
                        _ => ()
                    }
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
        let attack_per_sample = per_sample * (1.0 / self.params.attack_duration.get() as f64);
        let release_per_sample = per_sample * (1.0 / self.params.release_duration.get() as f64);


        let mut output_sample;
        for sample_idx in 0..samples {
            /**
             * Update the alpha...
             */
            for note in self.notes.iter_mut() {
                if !note.is_released && note.alpha < 1.0 {
                    note.alpha += attack_per_sample;
                }

                if note.is_released {
                    note.alpha -= release_per_sample;
                }
            }

            /**
             * ...and remove finished notes.
             */
            self.notes.retain(|n| n.alpha > 0.0);

            if !self.notes.is_empty() {
                let mut signal = 0.0;
                let params = self.params.deref();


                for note in &self.notes {
                    let point = [0.0, self.time * midi_pitch_to_freq(note.note)];

                    if params.a_white_noise.get() > 0.0 && note.alpha > 0.0001 {
                        signal += ((random::<f64>() - 0.5) * 2.0) * params.a_white_noise.get() as f64 * note.alpha;
                    }

                    if params.a_perlin.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_perlin.get(point) * params.a_perlin.get() as f64 * note.alpha;
                    }

                    if params.a_value.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_value.get(point) * params.a_value.get() as f64 * note.alpha;
                    }

                    if params.a_worley.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_worley.get(point) * params.a_worley.get() as f64 * note.alpha;
                    }

                    if params.a_ridged_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_ridged_multi.get(point) * params.a_ridged_multi.get() as f64 * note.alpha;
                    }

                    if params.a_open_simplex.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_open_simplex.get(point) * params.a_open_simplex.get() as f64 * note.alpha;
                    }

                    if params.a_billow.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_billow.get(point) * params.a_billow.get() as f64 * note.alpha;
                    }

                    if params.a_cylinders.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_cylinders.get(point) * params.a_cylinders.get() as f64 * note.alpha;
                    }

                    if params.a_hybrid_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_hybrid_multi.get(point) * params.a_hybrid_multi.get() as f64 * note.alpha;
                    }

                    if params.a_basic_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_basic_multi.get(point) * params.a_basic_multi.get() as f64 * note.alpha;
                    }
                }


                output_sample = signal as f32;
                self.time += per_sample;
            } else {
                output_sample = 0.0;
            }

            unsafe {
                LAST_SAMPLE = output_sample as f64;
            }


            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        let gui = vst_gui::new_plugin_gui(
            String::from(HTML),
            Box::new(move |message: String| {
                let mut tokens = message.split_whitespace();

                let command = tokens.next().unwrap_or("");
                let argument = tokens.next().unwrap_or("").parse::<f32>();

                let mut result = String::new();

                unsafe {
                    match command {
                        "getLast" => {
                            result = LAST_SAMPLE.to_string();
                        }
                        _ => {}
                    }
                }

                result
            }),
            Some((480, 320)));


        Some(Box::new(gui))
    }
}


plugin_main!(Whisper);



fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
}
