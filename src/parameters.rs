use vst::util::AtomicFloat;
use vst::plugin::PluginParameters;


pub struct WhisperParameters {
    // Amounts
    pub a_white_noise: AtomicFloat,
    pub a_perlin: AtomicFloat,
    pub a_value: AtomicFloat,
    pub a_worley: AtomicFloat,
    pub a_ridged_multi: AtomicFloat,
    pub a_open_simplex: AtomicFloat,
    pub a_billow: AtomicFloat,
    pub a_cylinders: AtomicFloat,
    pub a_hybrid_multi: AtomicFloat,
    pub a_basic_multi: AtomicFloat,

    pub attack_duration: AtomicFloat,
    pub release_duration: AtomicFloat,
}

impl PluginParameters for WhisperParameters {
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
