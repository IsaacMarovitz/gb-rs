use crate::memory::Memory;
use crate::sound::apu::{APU, DutyCycle};

pub struct SC1 {
    pub dac_enabled: bool,
    sweep_pace: u8,
    negative_direction: bool,
    sweep_step: u8,
    pub duty_cycle: DutyCycle,
    pub length_timer: u8,
    pub volume: u8,
    positive_envelope: bool,
    envelope_pace: u8,
    pub period: u16,
    pub trigger: bool,
    length_enabled: bool,
    length_cycle_count: u32,
    sweep_cycle_count: u32
}

impl SC1 {
    pub fn new() -> Self {
        Self {
            dac_enabled: false,
            sweep_pace: 0,
            negative_direction: false,
            sweep_step: 0,
            duty_cycle: DutyCycle::QUARTER,
            length_timer: 0,
            volume: 0,
            positive_envelope: false,
            envelope_pace: 0,
            period: 0,
            trigger: false,
            length_enabled: false,
            length_cycle_count: 0,
            sweep_cycle_count: 0
        }
    }

    pub fn clear(&mut self) {
        self.dac_enabled = false;
        self.sweep_pace = 0;
        self.negative_direction = false;
        self.sweep_step = 0;
        self.duty_cycle = DutyCycle::QUARTER;
        self.length_timer = 0;
        self.volume = 0;
        self.positive_envelope = false;
        self.envelope_pace = 0;
        self.period = 0;
        self.trigger = false;
        self.length_enabled = false;
    }

    pub fn cycle(&mut self, cycles: u32) {
        if self.length_enabled {
            self.length_cycle_count += cycles;

            if self.length_cycle_count >= APU::hz_to_cycles(256) {
                self.length_cycle_count = 0;

                if self.length_timer >= 64 {
                    println!("NOTE OVER");
                    self.dac_enabled = false;
                    self.length_enabled = false;
                } else {
                    self.length_timer += 1;
                }
            }
        }

        // if self.sweep_pace != 0 {
        //     self.sweep_cycle_count += cycles;
        //
        //     if self.sweep_cycle_count >= (APU::hz_to_cycles(128) * self.sweep_pace as u32) {
        //         self.sweep_cycle_count = 0;
        //
        //         let divisor = 2 ^ (self.sweep_step as u16);
        //         if divisor != 0 {
        //             let step = self.period / divisor;
        //             if self.negative_direction {
        //                 self.period -= step;
        //             } else {
        //                 let (value, overflow) = self.period.overflowing_add(step);
        //
        //                 if value > 0x7FF || overflow {
        //                     self.dac_enabled = false;
        //                     self.clear();
        //                 } else {
        //                     self.period = value;
        //                 }
        //             }
        //         }
        //     }
        // }
    }
}

impl Memory for SC1 {
    fn read(&self, a: u16) -> u8 {
        match a {
            // NR10: Sweep
            0xFF10 => (self.sweep_pace & 0b0000_0111) << 4 | (self.negative_direction as u8) << 3 | (self.sweep_step & 0b0000_0111) | 0x80,
            // NR11: Length Timer & Duty Cycle
            0xFF11 => (self.duty_cycle.bits()) << 6 | 0x3F,
            // NR12: Volume & Envelope
            0xFF12 => (self.volume & 0b0000_1111) << 4 | (self.positive_envelope as u8) << 3 | (self.envelope_pace & 0b0000_0111),
            // NR13: Period Low
            0xFF13 => 0xFF,
            // NR14: Period High & Control
            0xFF14 => (self.length_enabled as u8) << 6 | 0xBF,
            _ => 0xFF,
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            // NR10: Sweep
            0xFF10 => {
                self.sweep_pace = (v & 0b0111_0000) >> 4;
                self.negative_direction = ((v & 0b0000_1000) >> 3) != 0;
                self.sweep_step = v & 0b0000_0111;
            },
            // NR11: Length Timer & Duty Cycle
            0xFF11 => {
                self.duty_cycle = DutyCycle::from_bits_truncate(v >> 6);
                self.length_timer = v & 0b0011_1111;
            },
            // NR12: Volume & Envelope
            0xFF12 => {
                self.volume = (v & 0b1111_0000) >> 4;
                self.positive_envelope = ((v & 0b0000_1000) >> 3) != 0;
                self.envelope_pace = v & 0b0000_0111;

                if self.read(0xFF12) & 0xF8 != 0 {
                    self.dac_enabled = true;
                }
            },
            // NR13: Period Low
            0xFF13 => {
                self.period &= !0xFF;
                self.period |= v as u16;
            },
            // NR14: Period High & Control
            0xFF14 => {
                self.trigger = ((v & 0b1000_0000) >> 7) != 0;
                self.length_enabled = ((v & 0b0100_0000) >> 6) != 0;
                self.period &= 0b0000_0000_1111_1111;
                self.period |= ((v & 0b0000_0111) as u16) << 8;
            },
            _ => panic!("Write to unsupported SC1 address ({:#06x})!", a),
        }
    }
}