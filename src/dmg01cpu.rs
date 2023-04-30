mod apu;
mod cartridge;
mod joypad;
mod ppu;
mod timer;

use super::Log;

use apu::APU;
use cartridge::Cartridge;
use joypad::Joypad;
use ppu::PPU;
use timer::Timer;

const OPECODE_CYCLES: [u8; 256] = [
    // 0,1,2,3,4,5,6,7,8,9,a,b,c,d,e,f
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, // 0
    0, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4, 8, 4, // 1
    8, 12, 8, 8, 4, 4, 8, 4, 8, 8, 8, 8, 4, 4, 8, 4, // 2
    8, 12, 8, 8, 12, 12, 12, 4, 8, 8, 8, 8, 4, 4, 8, 4, // 3
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 4
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 5
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 6
    8, 8, 8, 8, 8, 8, 0, 8, 4, 4, 4, 4, 4, 4, 8, 4, // 7
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 8
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 9
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // a
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // b
    8, 12, 12, 16, 12, 16, 8, 16, 8, 16, 12, 0, 12, 24, 8, 16, // c
    8, 12, 12, 0, 12, 16, 8, 16, 8, 16, 12, 0, 12, 0, 8, 16, // d
    12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16, // e
    12, 12, 8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16, // f
];

const CB_OPECODE_CYCLES: [u8; 256] = [
    // 0,1,2,3,4,5,6,7,8,9,a,b,c,d,e,f
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 1
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 2
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 3
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 4
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 5
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 6
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 7
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 8
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 9
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // a
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // b
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // c
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // d
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // e
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // f
];

pub struct Dmg01Cpu {
    log_mode: u8,
    /* Clock */
    cycle: u8, // cpu clock 4.194304 MHz
    /* Interrupt */
    ime: u8, // interrupt master enable flag
    interrupt_enable: u8,
    interrupt_flag: u8,
    /* Halt */
    halt: u8, // cpu halt
    /* Registers */
    a: u8,   // accumulator
    f: u8,   // flag
    b: u8,   // general
    c: u8,   // general
    d: u8,   // general
    e: u8,   // general
    h: u8,   // general
    l: u8,   // general
    sp: u16, // stack pointer
    pc: u16, // program counter
    /* Memory */
    ram: [u8; 0x2000], // C000 - DFFF
    hram: [u8; 0x7f],  // FF80 - FFFE
    /* Peripheral */
    timer: Timer,
    pub ppu: PPU,
    pub apu: APU,
    pub joypad: Joypad,
    cartridge: Cartridge,
}

impl Dmg01Cpu {
    pub fn new(log_mode: u8, romfile: String) -> Self {
        let dmg01cpu = Dmg01Cpu {
            log_mode,
            cycle: 0,
            ime: 0x00,
            interrupt_flag: 0x00,
            interrupt_enable: 0x00,
            halt: 0,
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            sp: 0x0000,
            pc: 0x0100, // entry point
            ram: [0; 0x2000],
            hram: [0; 0x7f],
            timer: Timer::new(),
            apu: APU::new(log_mode),
            ppu: PPU::new(log_mode),
            joypad: Joypad::new(),
            cartridge: Cartridge::new(log_mode, romfile),
        };

        dmg01cpu
    }

    fn is_zero(value: u8) -> bool {
        if value == 0x00 {
            true
        } else {
            false
        }
    }

    /* Interrupt Master Enable Flag */
    fn get_ime(&self) -> bool {
        match self.ime {
            0x01 => true, // enable all interrupt
            _ => false,   // 0x00:disable all interrupt
        }
    }
    fn set_ime(&mut self, flag: bool) {
        self.ime = flag as u8;
    }

    /* CPU Halt */
    fn get_halt(&mut self) -> bool {
        match self.halt {
            0x01 => true,
            _ => false,
        }
    }
    fn set_halt(&mut self, flag: bool) {
        self.halt = flag as u8;
    }

    /* Zero Flag */
    fn set_z_zero(&mut self, flag: bool) {
        if flag {
            self.f = self.f | 0x80;
        } else {
            self.f = self.f & 0x7f;
        }
    }
    fn get_z_zero(&self) -> bool {
        if self.f & 0x80 == 0x80 {
            true
        } else {
            false
        }
    }

    /* Subtraction Flag (BCD) */
    fn set_n_subtraction(&mut self, flag: bool) {
        if flag {
            self.f = self.f | 0x40;
        } else {
            self.f = self.f & 0xbf;
        }
    }
    fn get_n_subtraction(&self) -> bool {
        if self.f & 0x40 == 0x40 {
            true
        } else {
            false
        }
    }

    /* Half Carry Flag (BCD) */
    fn set_h_half_carry(&mut self, flag: bool) {
        if flag {
            self.f = self.f | 0x20;
        } else {
            self.f = self.f & 0xdf;
        }
    }
    fn get_h_half_carry(&self) -> bool {
        if self.f & 0x20 == 0x20 {
            true
        } else {
            false
        }
    }

    /* Carry Flag */
    fn set_c_carry(&mut self, flag: bool) {
        if flag {
            self.f = self.f | 0x10;
        } else {
            self.f = self.f & 0xef;
        }
    }
    fn get_c_carry(&self) -> bool {
        if self.f & 0x10 == 0x10 {
            true
        } else {
            false
        }
    }

    /* 16-bit registers read & write */
    fn get_high(value: u16) -> u8 {
        (value >> 8) as u8
    }
    fn get_low(value: u16) -> u8 {
        (value & 0xff) as u8
    }
    fn make_16bit(high: u8, low: u8) -> u16 {
        (high as u16) << 8 | low as u16
    }

    fn get_af(&self) -> u16 {
        Self::make_16bit(self.a, self.f)
    }
    fn set_af(&mut self, value: u16) {
        self.a = Self::get_high(value);
        self.f = Self::get_low(value);
    }

    fn get_bc(&self) -> u16 {
        Self::make_16bit(self.b, self.c)
    }
    fn set_bc(&mut self, value: u16) {
        self.b = Self::get_high(value);
        self.c = Self::get_low(value);
    }

    fn get_de(&self) -> u16 {
        Self::make_16bit(self.d, self.e)
    }
    fn set_de(&mut self, value: u16) {
        self.d = Self::get_high(value);
        self.e = Self::get_low(value);
    }

    fn get_hl(&self) -> u16 {
        Self::make_16bit(self.h, self.l)
    }
    fn set_hl(&mut self, value: u16) {
        self.h = Self::get_high(value);
        self.l = Self::get_low(value);
    }

    /* 8-bit read and write */
    fn write(&mut self, address: u16, value: u8) {
        self.write_via_map(address, value);
    }

    fn read(&mut self, address: u16) -> u8 {
        let result: u8 = self.read_via_map(address);
        result
    }

    /* 16-bit read and write */
    fn write16(&mut self, address: u16, value: u16) {
        self.write(address, Self::get_low(value));
        self.write(address + 1, Self::get_high(value));
    }
    fn read16(&mut self, address: u16) -> u16 {
        let low: u8 = self.read(address);
        let high: u8 = self.read(address.wrapping_add(1));
        Self::make_16bit(high, low)
    }

    /// register or via memory map
    fn write_idx(&mut self, index: u8, value: u8) {
        match index {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => self.write(self.get_hl(), value),
            7 => self.a = value,
            _ => {
                panic!("unexpected index {:#04x}.", index)
            }
        }
    }
    /// register or via memory map
    fn read_idx(&mut self, index: u8) -> u8 {
        let result: u8 = match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.read(self.get_hl()),
            7 => self.a,
            _ => {
                panic!("unexpected index {:#04x}.", index)
            }
        };
        result
    }

    fn write16_regster(&mut self, index: u8, value: u16) {
        match index {
            0 => self.set_bc(value),
            1 => self.set_de(value),
            2 => self.set_hl(value),
            3 => self.sp = value,
            _ => {
                panic!("unexpected index {:#04x}.", index)
            }
        }
    }
    fn read16_regster(&self, index: u8) -> u16 {
        let result: u16 = match index {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.sp,
            _ => {
                panic!("unexpected index {:#04x}.", index)
            }
        };
        result
    }

    /* Program Counter */
    fn read_pc(&mut self) -> u8 {
        let result: u8 = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        result
    }
    fn read16_pc(&mut self) -> u16 {
        let result: u16 = self.read16(self.pc);
        self.pc = self.pc.wrapping_add(2);
        result
    }

    /* 8-bit Load Instructions */
    /// ld r,r
    /// ld r,(HL)
    /// ld (HL),r
    fn ld_rhl_rhl(&mut self, w_index: u8, r_index: u8) {
        let value: u8 = self.read_idx(r_index);

        self.write_idx(w_index, value);
    }

    /// ld r,n
    /// ld (HL),n
    fn ld_rhl_n(&mut self, index: u8) {
        let value: u8 = self.read_pc();

        self.write_idx(index, value);
    }

    /// ld A,(BC)
    fn ld_a_bc(&mut self) {
        self.a = self.read(self.get_bc());
    }

    /// ld A,(DE)
    fn ld_a_de(&mut self) {
        self.a = self.read(self.get_de());
    }

    /// ld A,(nn)
    fn ld_a_nn(&mut self) {
        let address: u16 = self.read16_pc();

        self.a = self.read(address);
    }

    /// ld (BC),A
    fn ld_bc_a(&mut self) {
        self.write(self.get_bc(), self.a);
    }

    /// ld (DE),A
    fn ld_de_a(&mut self) {
        self.write(self.get_de(), self.a);
    }

    /// ld (nn),A
    fn ld_nn_a(&mut self) {
        let address: u16 = self.read16_pc();

        self.write(address, self.a);
    }

    /// ld A,(FF00+n)
    fn ld_a_ff00n(&mut self) {
        let value: u16 = self.read_pc() as u16;
        let address: u16 = 0xff00 + value;

        self.a = self.read(address);
    }

    /// ld (FF00+n),A
    fn ld_ff00n_a(&mut self) {
        let value: u16 = self.read_pc() as u16;
        let address: u16 = 0xff00 + value;

        self.write(address, self.a);
    }

    /// ld A,(FF00+C)
    fn ld_a_ff00c(&mut self) {
        let address: u16 = 0xff00 + (self.c as u16);

        self.a = self.read(address);
    }

    /// ld (FF00+C),A
    fn ld_ff00c_a(&mut self) {
        let address: u16 = 0xff00 + (self.c as u16);

        self.write(address, self.a);
    }

    /// ldi (HL),A
    fn ldi_hl_a(&mut self) {
        self.write(self.get_hl(), self.a);
        self.set_hl(self.get_hl().wrapping_add(1));
    }

    /// ldi A,(HL)
    fn ldi_a_hl(&mut self) {
        self.a = self.read(self.get_hl());
        self.set_hl(self.get_hl().wrapping_add(1));
    }

    /// ldd (HL),A
    fn ldd_hl_a(&mut self) {
        self.write(self.get_hl(), self.a);
        self.set_hl(self.get_hl().wrapping_sub(1));
    }

    /// ldd A,(HL)
    fn ldd_a_hl(&mut self) {
        self.a = self.read(self.get_hl());
        self.set_hl(self.get_hl().wrapping_sub(1));
    }

    /* 16-bit Load instructions */

    /// ld rr,nn
    fn ld_rr_nn(&mut self, index: u8) {
        let value: u16 = self.read16_pc();
        self.write16_regster(index, value);
    }

    /// ld (nn),SP
    fn ld_nn_sp(&mut self) {
        let address: u16 = self.read16_pc();
        self.write16(address, self.sp);
    }

    /// ld SP,HL
    fn ld_sp_hl(&mut self) {
        self.sp = self.get_hl();
    }

    /// push rr
    fn push_rrbc(&mut self) {
        let value: u16 = self.get_bc();

        self.sp = self.sp.wrapping_sub(2);
        self.write16(self.sp, value)
    }
    /// push rr
    fn push_rrde(&mut self) {
        let value: u16 = self.get_de();

        self.sp = self.sp.wrapping_sub(2);
        self.write16(self.sp, value)
    }
    /// push rr
    fn push_rrhl(&mut self) {
        let value: u16 = self.get_hl();

        self.sp = self.sp.wrapping_sub(2);
        self.write16(self.sp, value)
    }
    /// push rr
    fn push_rraf(&mut self) {
        let value: u16 = self.get_af();

        self.sp = self.sp.wrapping_sub(2);
        self.write16(self.sp, value)
    }

    /// pop rr
    fn pop_rrbc(&mut self) {
        let value: u16 = self.read16(self.sp);

        self.set_bc(value);
        self.sp = self.sp.wrapping_add(2);
    }
    /// pop rr
    fn pop_rrde(&mut self) {
        let value: u16 = self.read16(self.sp);

        self.set_de(value);
        self.sp = self.sp.wrapping_add(2);
    }
    /// pop rr
    fn pop_rrhl(&mut self) {
        let value: u16 = self.read16(self.sp);

        self.set_hl(value);
        self.sp = self.sp.wrapping_add(2);
    }
    /// pop rr
    fn pop_rraf(&mut self) {
        // lower nibble of F is always 0
        let value: u16 = self.read16(self.sp) & 0xfff0;

        self.set_af(value);
        self.sp = self.sp.wrapping_add(2);
    }

    /* 8-bit Arithmetic/Logic instructions */

    fn add(&mut self, value: u8) {
        let (result, carry): (u8, bool) = self.a.overflowing_add(value);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        if (self.a & 0x0f) + (value & 0x0f) > 0x0f {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        self.set_c_carry(carry);

        self.a = result;
    }

    /// add A,r
    /// add A,(HL)
    fn add_a_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);

        self.add(value);
    }

    /// add A,n
    fn add_a_n(&mut self) {
        let value: u8 = self.read_pc();

        self.add(value);
    }

    fn adc(&mut self, value: u8) {
        let carry: u8 = self.get_c_carry() as u8;
        let result: u8 = self.a.wrapping_add(value).wrapping_add(carry);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        if (self.a & 0x0f) + (value & 0x0f) + carry > 0x0f {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        if (self.a as u16) + (value as u16) + (carry as u16) > 0x00ff {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }

        self.a = result;
    }

    /// adc A,r
    /// adc A,(HL)
    fn adc_a_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);

        self.adc(value);
    }

    /// adc A,n
    fn adc_a_n(&mut self) {
        let value: u8 = self.read_pc();

        self.adc(value);
    }

    fn sub(&mut self, value: u8) {
        let (result, carry): (u8, bool) = self.a.overflowing_sub(value);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(true);
        if (self.a & 0x0f) < (value & 0x0f) {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        self.set_c_carry(carry);

        self.a = result;
    }

    /// sub r
    /// sub (HL)
    fn sub_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);

        self.sub(value);
    }

    /// sub n
    fn sub_n(&mut self) {
        let value: u8 = self.read_pc();

        self.sub(value);
    }

    fn sbc(&mut self, value: u8) {
        let carry: u8 = self.get_c_carry() as u8;
        let result: u8 = self.a.wrapping_sub(value).wrapping_sub(carry);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(true);
        if (self.a & 0x0f) < (value & 0x0f) + carry {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        if (self.a as u16) < (value as u16) + (carry as u16) {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }

        self.a = result;
    }

    /// sbc A,r
    /// sbc A,(HL)
    fn sbc_a_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);

        self.sbc(value)
    }

    /// sbc A,n
    fn sbc_a_n(&mut self) {
        let value: u8 = self.read_pc();

        self.sbc(value);
    }

    /// and r
    /// and (HL)
    fn and_rhl(&mut self, index: u8) {
        self.a = self.a & self.read_idx(index);

        self.set_z_zero(Self::is_zero(self.a));
        self.set_n_subtraction(false);
        self.set_h_half_carry(true);
        self.set_c_carry(false);
    }

    /// and n
    fn and_n(&mut self) {
        let value = self.read_pc();

        self.a = self.a & value;

        self.set_z_zero(Self::is_zero(self.a));
        self.set_n_subtraction(false);
        self.set_h_half_carry(true);
        self.set_c_carry(false)
    }

    /// xor r
    /// xor (HL)
    fn xor_rhl(&mut self, index: u8) {
        self.a = self.a ^ self.read_idx(index);

        self.set_z_zero(Self::is_zero(self.a));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(false);
    }

    /// xor n
    fn xor_n(&mut self) {
        let value = self.read_pc();

        self.a = self.a ^ value;

        self.set_z_zero(Self::is_zero(self.a));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(false)
    }

    /// or r
    /// or (HL)
    fn or_rhl(&mut self, index: u8) {
        self.a = self.a | self.read_idx(index);

        self.set_z_zero(Self::is_zero(self.a));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(false);
    }

    /// or n
    fn or_n(&mut self) {
        let value = self.read_pc();

        self.a = self.a | value;

        self.set_z_zero(Self::is_zero(self.a));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(false)
    }

    /// cp r
    /// cp (HL)
    fn cp_rhl(&mut self, index: u8) {
        // compare
        let value: u8 = self.read_idx(index);

        let (result, carry) = self.a.overflowing_sub(value);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(true);
        if self.a & 0x0f < value & 0x0f {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        self.set_c_carry(carry);
    }

    /// cp n
    fn cp_n(&mut self) {
        // compare
        let value = self.read_pc();

        let (result, carry) = self.a.overflowing_sub(value);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(true);
        if self.a & 0x0f < value & 0x0f {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        self.set_c_carry(carry);
    }

    /// inc r
    /// inc (HL)
    fn inc_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = value.wrapping_add(1);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        if value & 0x0f == 0x0f {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
    }

    /// dec r
    /// dec (HL)
    fn dec_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = value.wrapping_sub(1);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(true);
        if value & 0x0f == 0x00 {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
    }

    /// daa
    fn daa(&mut self) {
        // decimal adjust after addtion

        if self.get_n_subtraction() {
            if self.get_c_carry() {
                self.a = self.a.wrapping_sub(0x60);
            }
            if self.get_h_half_carry() {
                self.a = self.a.wrapping_sub(0x06);
            }
        } else {
            if self.get_c_carry() || self.a > 0x99 {
                self.a = self.a.wrapping_add(0x60);
                self.set_c_carry(true);
            }
            if self.get_h_half_carry() || self.a & 0x0f > 0x09 {
                self.a = self.a.wrapping_add(0x06);
            }
        }

        self.set_z_zero(Self::is_zero(self.a));
        self.set_h_half_carry(false);
    }

    /// cpl
    fn cpl(&mut self) {
        // complement accumulator
        self.a = !self.a;
        self.set_n_subtraction(true);
        self.set_h_half_carry(true);
    }

    /* 16-bit Arithmetic/Logic instructions */

    /// add HL,rr
    fn add_hl_rr(&mut self, index: u8) {
        let value_hl: u16 = self.get_hl();
        let value: u16 = self.read16_regster(index);

        let (result, carry): (u16, bool) = value_hl.overflowing_add(value);
        self.set_hl(result);

        self.set_n_subtraction(false);
        if (value_hl & 0x0fff) + (value & 0x0fff) > 0x0fff {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        self.set_c_carry(carry);
    }

    /// inc rr
    fn inc_rr(&mut self, index: u8) {
        let value: u16 = self.read16_regster(index).wrapping_add(1);

        self.write16_regster(index, value);
    }

    /// dec rr
    fn dec_rr(&mut self, index: u8) {
        let value: u16 = self.read16_regster(index).wrapping_sub(1);

        self.write16_regster(index, value);
    }

    fn add_sp(&mut self, signed_num: i8) -> u16 {
        let value: u16 = signed_num as u16;

        let result: u16 = self.sp.wrapping_add(value);

        self.set_z_zero(false);
        self.set_n_subtraction(false);
        if (self.sp & 0x000f) + (value & 0x000f) > 0x000f {
            self.set_h_half_carry(true);
        } else {
            self.set_h_half_carry(false);
        }
        if (self.sp & 0x00ff) + (value & 0x00ff) > 0x00ff {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }

        result
    }

    /// add SP,dd
    fn add_sp_dd(&mut self) {
        let value: i8 = self.read_pc() as i8;

        self.sp = self.add_sp(value);
    }

    /// ld HL,SP+dd
    fn ld_hl_spdd(&mut self) {
        let value: i8 = self.read_pc() as i8;

        let result: u16 = self.add_sp(value);

        self.set_hl(result);
    }

    /* Rotate and Shift instructions */

    /// rlca
    fn rlca(&mut self) {
        self.rlc(7); // 7:register a
        self.set_z_zero(false);
    }

    /// rla
    fn rla(&mut self) {
        self.rl(7); // 7:register a
        self.set_z_zero(false);
    }

    /// rrca
    fn rrca(&mut self) {
        self.rrc(7); // 7:register a
        self.set_z_zero(false);
    }

    /// rra
    fn rra(&mut self) {
        self.rr(7); // 7:register a
        self.set_z_zero(false);
    }

    fn rlc(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = value.rotate_left(1);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value >> 7 & 0x01 == 0x01 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /// rlc r
    /// rlc (HL)
    fn rlc_rhl(&mut self, index: u8) {
        self.rlc(index);
    }

    fn rl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = (value << 1) | (self.get_c_carry() as u8);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value >> 7 & 0x01 == 0x01 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /// rl r
    /// rl (HL)
    fn rl_rhl(&mut self, index: u8) {
        self.rl(index);
    }

    fn rrc(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = value.rotate_right(1);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value & 0x01 == 0x01 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /// rrc r
    /// rrc (HL)
    fn rrc_rhl(&mut self, index: u8) {
        self.rrc(index);
    }

    fn rr(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = (value >> 1) | (self.get_c_carry() as u8) << 7;

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value & 0x01 == 0x01 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /// rr r
    /// rr (HL)
    fn rr_rhl(&mut self, index: u8) {
        self.rr(index);
    }

    /// sla r
    /// sla (HL)
    fn sla_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = value << 1;

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value & 0x80 == 0x80 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /// swap r
    /// swap (HL)
    fn swap_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = ((value & 0x0f) << 4) | ((value & 0xf0) >> 4);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(false);
    }

    /// sra r
    /// sra (HL)
    fn sra_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = (value >> 1) | (value & 0x80);

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value & 0x01 == 0x01 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /// srl r
    /// srl (HL)
    fn srl_rhl(&mut self, index: u8) {
        let value: u8 = self.read_idx(index);
        let result: u8 = value >> 1;

        self.write_idx(index, result);

        self.set_z_zero(Self::is_zero(result));
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        if value & 0x01 == 0x01 {
            self.set_c_carry(true);
        } else {
            self.set_c_carry(false);
        }
    }

    /* Single-bit Operation instructions */

    /// bit n,r
    /// bit n,(HL)
    fn bit_n_rhl(&mut self, offset: u8, index: u8) {
        // test bit
        if (self.read_idx(index) >> offset & 0x01) == 0 {
            self.set_z_zero(true);
        } else {
            self.set_z_zero(false);
        }
        self.set_n_subtraction(false);
        self.set_h_half_carry(true);
    }

    /// set n,r
    /// set n,(HL)
    fn set_n_rhl(&mut self, offset: u8, index: u8) {
        // set bit
        let value = self.read_idx(index);

        self.write_idx(index, value | (0x01 << offset));
    }

    /// res n,r
    /// res n,(HL)
    fn res_n_rhl(&mut self, offset: u8, index: u8) {
        // reset bit
        let value = self.read_idx(index);

        self.write_idx(index, value & !(0x01 << offset));
    }

    /* CPU Control instructions */

    /// ccf
    fn ccf(&mut self) {
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(!self.get_c_carry());
    }

    /// scf
    fn scf(&mut self) {
        self.set_n_subtraction(false);
        self.set_h_half_carry(false);
        self.set_c_carry(true);
    }

    /// nop
    fn nop(&self) {
        // no operation
    }

    /// halt
    fn halt(&mut self) {
        self.set_halt(true);
    }

    /// stop

    /// di
    fn di(&mut self) {
        // disable interrupts
        self.set_ime(false);
    }

    /// ei
    fn ei(&mut self) {
        // enable interrupts
        self.set_ime(true);
    }

    /* Jump instructions */

    fn conditional(&self, index: u8) -> bool {
        match index {
            0 => !self.get_z_zero(),
            1 => self.get_z_zero(),
            2 => !self.get_c_carry(),
            3 => self.get_c_carry(),
            _ => {
                panic!("unexpected index {:#04x}.", index)
            }
        }
    }

    fn jp(&mut self, address: u16) {
        // jump
        self.pc = address;
    }

    /// jp nn
    fn jp_nn(&mut self) {
        let address: u16 = self.read16_pc();
        self.jp(address);
    }

    /// jp HL
    fn jp_hl(&mut self) {
        self.pc = self.get_hl();
    }

    /// jp f,nn
    fn jp_f_nn(&mut self, index: u8) {
        let address: u16 = self.read16_pc();

        if self.conditional(index) {
            self.cycle += 4;
            self.jp(address);
        }
    }

    fn jr(&mut self, offset: i8) {
        // relative jump
        self.pc = self.pc.wrapping_add(offset as u16);
    }

    /// jr PC+dd
    fn jr_pcdd(&mut self) {
        let offset: i8 = self.read_pc() as i8;

        self.jr(offset);
    }

    /// jr f,PC+dd
    fn jr_f_pcdd(&mut self, index: u8) {
        let offset: i8 = self.read_pc() as i8;

        if self.conditional(index) {
            self.cycle += 4;
            self.jr(offset);
        }
    }

    fn call(&mut self, address: u16) {
        self.sp = self.sp.wrapping_sub(2);

        self.write16(self.sp, self.pc);
        self.pc = address;
    }

    /// call nn
    fn call_nn(&mut self) {
        let address: u16 = self.read16_pc();

        self.call(address);
    }

    /// call f,nn
    fn call_f_nn(&mut self, index: u8) {
        let address: u16 = self.read16_pc();

        if self.conditional(index) {
            self.cycle += 12;
            self.call(address);
        }
    }

    /// ret
    fn ret(&mut self) {
        let value: u16 = self.read16(self.sp);

        self.pc = value;
        self.sp = self.sp.wrapping_add(2);
    }

    /// ret f
    fn ret_f(&mut self, index: u8) {
        if self.conditional(index) {
            self.cycle += 12;
            self.ret();
        }
    }

    /// reti
    fn reti(&mut self) {
        // return and enable interrupts
        self.set_ime(true);
        self.ret();
    }

    /// rst n
    fn rst_n(&mut self, address: u8) {
        // restart
        self.call(address as u16);
    }

    /* Other */

    fn cb_prefix(&mut self) {
        let opecode: u8 = self.read_pc();
        self.cycle += CB_OPECODE_CYCLES[opecode as usize];
        let opecode345: u8 = opecode >> 3 & 0x07;
        let opecode012: u8 = opecode & 0x07;

        let text = format!("a={:#04x} f={:#04x} b={:#04x} c={:#04x} d={:#04x} e={:#04x} h={:#04x} l={:#04x} sp={:#08x} pc={:#08x} op={:#04x}",self.a,self.f,self.b,self.c,self.d,self.e,self.h,self.l,self.sp,self.pc,opecode);
        Log::cpu(text, self.log_mode);

        match opecode {
            0x00..=0x07 => self.rlc_rhl(opecode012),
            0x08..=0x0f => self.rrc_rhl(opecode012),
            0x10..=0x17 => self.rl_rhl(opecode012),
            0x18..=0x1f => self.rr_rhl(opecode012),
            0x20..=0x27 => self.sla_rhl(opecode012),
            0x28..=0x2f => self.sra_rhl(opecode012),
            0x30..=0x37 => self.swap_rhl(opecode012),
            0x38..=0x3f => self.srl_rhl(opecode012),
            0x40..=0x7f => self.bit_n_rhl(opecode345, opecode012),
            0x80..=0xbf => self.res_n_rhl(opecode345, opecode012),
            0xc0..=0xff => self.set_n_rhl(opecode345, opecode012),
        }
    }

    fn fetch_execute(&mut self) {
        let opecode: u8 = self.read_pc();
        self.cycle += OPECODE_CYCLES[opecode as usize];

        let text = format!("a={:#04x} f={:#04x} b={:#04x} c={:#04x} d={:#04x} e={:#04x} h={:#04x} l={:#04x} sp={:#08x} pc={:#08x} op={:#04x}",self.a,self.f,self.b,self.c,self.d,self.e,self.h,self.l,self.sp,self.pc,opecode);
        Log::cpu(text, self.log_mode);

        let opecode012: u8 = opecode & 0x07;
        let opecode345: u8 = opecode >> 3 & 0x07;

        match opecode {
            0x00 => self.nop(),
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_rr_nn(opecode >> 4),
            0x08 => self.ld_nn_sp(),
            0xf9 => self.ld_sp_hl(),
            0x02 => self.ld_bc_a(),
            0x12 => self.ld_de_a(),
            0x0a => self.ld_a_bc(),
            0x1a => self.ld_a_de(),

            0xc5 => self.push_rrbc(),
            0xd5 => self.push_rrde(),
            0xe5 => self.push_rrhl(),
            0xf5 => self.push_rraf(),

            0xc1 => self.pop_rrbc(),
            0xd1 => self.pop_rrde(),
            0xe1 => self.pop_rrhl(),
            0xf1 => self.pop_rraf(),

            0xc2 | 0xd2 | 0xca | 0xda => self.jp_f_nn(opecode345),
            0xc3 => self.jp_nn(),
            0xe9 => self.jp_hl(),
            0x20 | 0x30 | 0x28 | 0x38 => self.jr_f_pcdd(opecode345 - 4),
            0x18 => self.jr_pcdd(),

            0x07 => self.rlca(),
            0x17 => self.rla(),
            0x0f => self.rrca(),
            0x1f => self.rra(),

            0x09 | 0x19 | 0x29 | 0x39 => self.add_hl_rr(opecode >> 4),
            0xe8 => self.add_sp_dd(),
            0xf8 => self.ld_hl_spdd(),

            0x80..=0x87 => self.add_a_rhl(opecode012),
            0x88..=0x8f => self.adc_a_rhl(opecode012),
            0x90..=0x97 => self.sub_rhl(opecode012),
            0x98..=0x9f => self.sbc_a_rhl(opecode012),
            0xa0..=0xa7 => self.and_rhl(opecode012),
            0xb0..=0xb7 => self.or_rhl(opecode012),
            0xa8..=0xaf => self.xor_rhl(opecode012),
            0xb8..=0xbf => self.cp_rhl(opecode012),

            0x27 => self.daa(),

            0x2f => self.cpl(),

            0x37 => self.scf(),
            0x3f => self.ccf(),

            0xc6 => self.add_a_n(),
            0xd6 => self.sub_n(),
            0xe6 => self.and_n(),
            0xf6 => self.or_n(),
            0xce => self.adc_a_n(),
            0xde => self.sbc_a_n(),
            0xee => self.xor_n(),
            0xfe => self.cp_n(),

            0x22 => self.ldi_hl_a(),
            0x32 => self.ldd_hl_a(),
            0x2a => self.ldi_a_hl(),
            0x3a => self.ldd_a_hl(),

            0xe0 => self.ld_ff00n_a(),
            0xf0 => self.ld_a_ff00n(),
            0xe2 => self.ld_ff00c_a(),
            0xf2 => self.ld_a_ff00c(),

            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => self.ld_rhl_n(opecode345),

            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => self.inc_rhl(opecode345),

            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => self.dec_rhl(opecode345),

            0x40..=0x75 | 0x77..=0x7f => self.ld_rhl_rhl(opecode345, opecode012),

            0xea => self.ld_nn_a(),
            0xfa => self.ld_a_nn(),

            0x03 | 0x13 | 0x23 | 0x33 => self.inc_rr(opecode >> 4),
            0x0b | 0x1b | 0x2b | 0x3b => self.dec_rr(opecode >> 4),

            0xcd => self.call_nn(),

            0xc4 | 0xd4 | 0xcc | 0xdc => self.call_f_nn(opecode345),

            0xc9 => self.ret(),

            0xc0 | 0xd0 | 0xc8 | 0xd8 => self.ret_f(opecode345),

            0xd9 => self.reti(),

            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => self.rst_n(opecode - 0xc7),

            0xf3 => self.di(),
            0xfb => self.ei(),

            0xcb => self.cb_prefix(), // operation extention

            0x76 => self.halt(),
            _ => {
                panic!("unexpected opecode {:#04x}", opecode)
            }
        }
    }

    fn call_isr(&mut self, index: u8) {
        self.interrupt_flag &= !(0x01 << index);
        self.set_ime(false);
        self.set_halt(false);
        let isr: u16 = match index {
            0 => 0x40,
            1 => 0x48,
            2 => 0x50,
            3 => 0x80,
            4 => 0x70,
            _ => panic!("Invalid IRQ index {}", index),
        };

        self.cycle += 24;

        self.call(isr);
    }

    fn update_irqs(&mut self) {
        for i in 0..5 {
            let iflag = self.interrupt_flag & (0x01 << i) > 0;
            let ienable = self.interrupt_enable & (0x01 << i) > 0;

            if iflag && ienable {
                self.call_isr(i);
                break;
            }
        }
    }

    pub fn execute(&mut self) -> u8 {
        let mut total_cycle: u8 = 0;

        self.cycle = 0;

        if self.get_halt() {
            self.cycle += 4;
        } else {
            self.fetch_execute();
        }

        total_cycle += self.cycle;

        self.update_device();

        if self.interrupt_flag & self.interrupt_enable & 0x1f > 0 {
            self.set_halt(false);
            if self.get_ime() {
                self.cycle = 0;
                self.update_irqs();
                self.update_device();

                total_cycle += self.cycle;
            } else {
                // halt bug
            }
        }

        total_cycle
    }

    /// LCD OAM DMA Transfers
    fn dma_transfer(&mut self, address: u8) {
        if 0x80 <= address && address <= 0xdf {
            let read_mask: u16 = (address as u16) << 8;
            const WRITE_MASK: u16 = 0xfe00;

            for i in 0..0xa0 {
                let value = self.read_via_map(read_mask | i); // source xx00-xx9F
                self.write_via_map(WRITE_MASK | i, value); // destination  FE00-FE9F
            }
        } else {
            panic!("invalid DMA source address {:#04x}", address);
        }
    }

    /// Use the memory map
    fn read_via_map(&self, address: u16) -> u8 {
        Log::io(
            format!("{: <15}:{:#04x}", "read address", address),
            self.log_mode,
        );

        let result: u8 = match address {
            0x0000..=0x7fff => self.cartridge.read(address),
            0x8000..=0x9fff => self.ppu.read(address), // vram
            0xa000..=0xbfff => self.cartridge.read(address),
            0xc000..=0xdfff => self.ram[(address & 0x1fff) as usize],
            0xe000..=0xfdff => self.ram[((address - 0x2000) & 0x1fff) as usize],
            0xfe00..=0xfe9f => self.ppu.read(address), // sprite
            0xfea0..=0xfeff => 0xff,                   // not usable
            0xff00 => self.joypad.read(address),
            0xff04..=0xff07 => self.timer.read(address),
            0xff0f => self.interrupt_flag,
            0xff10..=0xff26 => self.apu.read(address),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.read(address), // lcd
            0xff80..=0xfffe => self.hram[(address & 0x007f) as usize],
            0xffff => self.interrupt_enable,
            _ => {
                0xff
                /*
                panic!(
                    "unexpected address {:#08x}.ram is disabled.need 0xff?",
                    address
                )
                */
            }
        };

        Log::io(format!("{: <15}:{:#04x}", "result", result), self.log_mode);
        result
    }

    /// Use the memory map
    fn write_via_map(&mut self, address: u16, value: u8) {
        Log::io(
            format!("{: <15}:{:#04x}", "write address", address),
            self.log_mode,
        );
        Log::io(format!("{: <15}:{:#04x}", "value", value), self.log_mode);

        match address {
            0x0000..=0x7fff => self.cartridge.write(address, value),
            0x8000..=0x9fff => self.ppu.write(address, value), // vram
            0xa000..=0xbfff => self.cartridge.write(address, value),
            0xc000..=0xdfff => self.ram[(address & 0x1fff) as usize] = value,
            0xe000..=0xfdff => self.ram[((address - 0x2000) & 0x1fff) as usize] = value,
            0xfe00..=0xfe9f => self.ppu.write(address, value), // sprite
            0xfea0..=0xfeff => (),                             // not usable
            0xff00 => self.joypad.write(address, value),
            0xff04..=0xff07 => self.timer.write(address, value),
            0xff0f => self.interrupt_flag = value,
            0xff10..=0xff3f => self.apu.write(address, value),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.write(address, value), // lcd
            0xff46 => self.dma_transfer(value),
            0xff80..=0xfffe => self.hram[(address & 0x007f) as usize] = value,
            0xffff => self.interrupt_enable = value,
            _ => {
                // panic!("unexpected address {:#08x}", address)
            }
        }
    }

    fn update_device(&mut self) {
        self.ppu.update(self.cycle);
        self.timer.update(self.cycle);

        if self.ppu.irq_vblank {
            self.interrupt_flag |= 0x01;
            self.ppu.irq_vblank = false;
        }

        if self.ppu.irq_lcdc {
            self.interrupt_flag |= 0x02;
            self.ppu.irq_lcdc = false;
        }

        if self.timer.irq {
            self.interrupt_flag |= 0x04;
            self.timer.irq = false;
        }

        if self.joypad.irq {
            self.interrupt_flag |= 0x10;
            self.joypad.irq = false;
        }
    }
}
