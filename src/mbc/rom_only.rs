use crate::mbc::mode::MBC;
use crate::memory::Memory;

pub struct ROMOnly {
    rom: Vec<u8>
}

impl Memory for ROMOnly {
    fn read(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x7FFF => self.rom[a as usize],
            _ => panic!("Read to unsupported ROM-only address ({:#06x})!", a),
        }
    }

    fn write(&mut self, a: u16, v: u8) { }
}

impl MBC for ROMOnly { }

impl ROMOnly {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom
        }
    }
}