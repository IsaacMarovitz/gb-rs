use bitflags::bitflags;
use crate::memory::Memory;
use crate::sound::sc1::SC1;
use crate::sound::sc2::SC2;
use crate::sound::sc3::{OutputLevel, SC3};
use crate::sound::sc4::SC4;
use crate::sound::synth::Synth;

pub struct APU {
    audio_enabled: bool,
    is_ch_4_on: bool,
    is_ch_3_on: bool,
    is_ch_2_on: bool,
    is_ch_1_on: bool,
    left_volume: u8,
    right_volume: u8,
    panning: Panning,
    sc1: SC1,
    sc2: SC2,
    sc3: SC3,
    sc4: SC4,
    synth: Synth
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct Panning: u8 {
        const CH4_LEFT = 0b1000_0000;
        const CH3_LEFT = 0b0100_0000;
        const CH2_LEFT = 0b0010_0000;
        const CH1_LEFT = 0b0001_0000;
        const CH4_RIGHT = 0b0000_1000;
        const CH3_RIGHT = 0b0000_0100;
        const CH2_RIGHT = 0b0000_0010;
        const CH1_RIGHT = 0b0000_0001;
    }
}

impl APU {
    pub fn new() -> Self {
        let synth = Synth::new();

        Self {
            audio_enabled: true,
            is_ch_4_on: false,
            is_ch_3_on: false,
            is_ch_2_on: false,
            is_ch_1_on: false,
            left_volume: 0,
            right_volume: 0,
            panning: Panning::empty(),
            sc1: SC1::new(),
            sc2: SC2::new(),
            sc3: SC3::new(),
            sc4: SC4::new(),
            synth
        }
    }

    pub fn cycle(&mut self, cycles: u32) {
        self.sc1.cycle(cycles);
        self.sc2.cycle(cycles);
        self.sc3.cycle(cycles);
        self.sc4.cycle(cycles);

        let s1_vol = {
            if self.sc1.dac_enabled {
                self.sc1.volume as f64 / 0xF as f64
            } else {
                0.0
            }
        };

        let s1_duty = {
            match self.sc1.duty_cycle {
                DutyCycle::EIGHTH => 0.125,
                DutyCycle::QUARTER => 0.25,
                DutyCycle::HALF => 0.5,
                DutyCycle::THREE_QUARTERS => 0.75,
                _ => 0.0
            }
        };

        let s2_vol = {
            if self.sc2.dac_enabled {
                self.sc2.volume as f64 / 0xF as f64
            } else {
                0.0
            }
        };

        let s2_duty = {
            match self.sc2.duty_cycle {
                DutyCycle::EIGHTH => 0.125,
                DutyCycle::QUARTER => 0.25,
                DutyCycle::HALF => 0.5,
                DutyCycle::THREE_QUARTERS => 0.75,
                _ => 0.0
            }
        };

        let s3_vol = {
            if self.sc3.dac_enabled {
                match self.sc3.output_level {
                    OutputLevel::MUTE => 0.0,
                    OutputLevel::QUARTER => 0.25,
                    OutputLevel::HALF => 0.5,
                    OutputLevel::MAX => 1.0,
                    _ => 0.0
                }
            } else {
                0.0
            }
        };

        let s4_vol = {
            if self.sc4.dac_enabled {
                self.sc4.final_volume as f64 / 0xF as f64
            } else {
                0.0
            }
        };

        // TODO: Amplifier on original hardware NEVER completely mutes non-silent input
        let global_l = {
            if self.audio_enabled {
                self.left_volume as f64 / 0xF as f64
            } else {
                0.0
            }
        };

        let global_r = {
            if self.audio_enabled {
                self.right_volume as f64 / 0xF as f64
            } else {
                0.0
            }
        };

        self.synth.s1_freq.set_value(131072.0 / (2048.0 - self.sc1.period as f64));
        self.synth.s1_vol.set_value(s1_vol);
        self.synth.s1_duty.set_value(s1_duty);
        self.synth.s1_l.set_value(if self.panning.contains(Panning::CH1_LEFT) { 1.0 } else { 0.0 });
        self.synth.s1_r.set_value(if self.panning.contains(Panning::CH1_RIGHT) { 1.0 } else { 0.0 });

        self.synth.s2_freq.set_value(131072.0 / (2048.0 - self.sc2.period as f64));
        self.synth.s2_vol.set_value(s2_vol);
        self.synth.s2_duty.set_value(s2_duty);
        self.synth.s2_l.set_value(if self.panning.contains(Panning::CH2_LEFT) { 1.0 } else { 0.0 });
        self.synth.s2_r.set_value(if self.panning.contains(Panning::CH2_RIGHT) { 1.0 } else { 0.0 });

        self.synth.s3_freq.set_value(65536.0 / (2048.0 - self.sc3.period as f64));
        self.synth.s3_vol.set_value(s3_vol);
        self.synth.s3_l.set_value(if self.panning.contains(Panning::CH3_LEFT) { 1.0 } else { 0.0 });
        self.synth.s3_r.set_value(if self.panning.contains(Panning::CH3_RIGHT) { 1.0 } else { 0.0 });

        self.synth.s4_freq.set_value(self.sc4.frequency as f64);
        self.synth.s4_vol.set_value(s4_vol);
        self.synth.s4_l.set_value(if self.panning.contains(Panning::CH4_LEFT) { 1.0 } else { 0.0 });
        self.synth.s4_r.set_value(if self.panning.contains(Panning::CH4_RIGHT) { 1.0 } else { 0.0 });

        self.synth.global_l.set_value(global_l);
        self.synth.global_r.set_value(global_r);
    }

    pub fn hz_to_cycles(hz: u32) -> u32 {
        let gameboy_freq = 4 * 1024 * 1024;
        return gameboy_freq / hz;
    }
}

impl Memory for APU {
    fn read(&self, a: u16) -> u8 {
        match a {
            // NR52: Audio Master Control
            0xFF26 => ((self.audio_enabled as u8) << 7) |
                      ((self.is_ch_4_on as u8) << 3) |
                      ((self.is_ch_3_on as u8) << 2) |
                      ((self.is_ch_2_on as u8) << 1) |
                      ((self.is_ch_1_on as u8) << 0) | 0x70,
            // NR51: Sound Panning
            0xFF25 => self.panning.bits(),
            // NR50: Master Volume & VIN
            0xFF24 => (self.left_volume & 0b0000_0111) << 4 |
                      (self.right_volume & 0b0000_0111),
            0xFF10..=0xFF14 => self.sc1.read(a),
            0xFF15..=0xFF19 => self.sc2.read(a),
            0xFF1A..=0xFF1E => self.sc3.read(a),
            0xFF30..=0xFF3F => self.sc3.read(a),
            0xFF20..=0xFF24 => self.sc4.read(a),
            _ => 0xFF
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        let mut set_apu_control = false;

        match a {
            // NR52: Audio Master Control
            0xFF26 => {
                set_apu_control = true;
                self.audio_enabled = (v >> 7) == 0x01;
            },
            // NR51: Sound Panning
            0xFF25 => {
                if self.audio_enabled {
                    self.panning = Panning::from_bits_truncate(v)
                }
            },
            // NR50: Master Volume & VIN
            0xFF24 => {
                if self.audio_enabled {
                    self.left_volume = v >> 4;
                    self.right_volume = v & 0b0000_0111;
                }
            },
            0xFF10..=0xFF14 => {
                if self.audio_enabled {
                    self.sc1.write(a, v)
                }
            },
            0xFF16..=0xFF19 => {
                if self.audio_enabled {
                    self.sc2.write(a, v)
                }
            },
            0xFF1A..=0xFF1E => {
                if self.audio_enabled {
                    self.sc3.write(a, v)
                }
            },
            0xFF30..=0xFF3F => self.sc3.write(a, v),
            0xFF20..=0xFF24 => {
                if self.audio_enabled {
                    self.sc4.write(a, v)
                }
            },
            _ => ()
            // _ => panic!("Write to unsupported APU address ({:#06x})!", a),
        }

        if self.sc1.trigger {
            self.sc1.trigger = false;
            if self.sc1.dac_enabled {
                self.is_ch_1_on = true;
            }
        }

        if self.sc2.trigger {
            self.sc2.trigger = false;
            if self.sc2.dac_enabled {
                self.is_ch_2_on = true;
            }
        }

        if self.sc3.trigger {
            self.sc3.trigger = false;
            if self.sc3.dac_enabled {
                self.is_ch_3_on = true;
            }
        }

        if self.sc4.trigger {
            self.sc4.trigger = false;
            self.sc4.lfsr = 0;
            if self.sc4.dac_enabled {
                self.is_ch_4_on = true;
            }
        }

        if set_apu_control {
            if !self.audio_enabled {
                self.is_ch_1_on = false;
                self.is_ch_2_on = false;
                self.is_ch_3_on = false;
                self.is_ch_4_on = false;
                self.left_volume = 0;
                self.right_volume = 0;

                self.panning = Panning::empty();

                self.sc1.clear();
                self.sc2.clear();
                self.sc3.clear();
                self.sc4.clear();
            }
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct DutyCycle: u8 {
        const EIGHTH = 0b0000_0000;
        const QUARTER = 0b0000_0001;
        const HALF = 0b0000_00010;
        const THREE_QUARTERS = 0b0000_0011;
    }
}