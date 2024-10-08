use std::{fmt::{self, Display, Formatter}, thread::sleep, time::Duration};

use crate::core::memory;

use super::constants::{INTERRPUT_LIST, INTERRUPT_ENABLE, INTERRUPT_FLAG};
pub enum RegisterValue {
    A(u8),
    F(u8),
    B(u8),
    C(u8),
    D(u8),
    E(u8),
    H(u8),
    L(u8),
    SP(u16),
    PC(u16),
    AF(u16),
    BC(u16),
    DE(u16),
    HL(u16),
}
pub const FLAG_Z_BIT: u8 = 1 << 7;
pub const FLAG_N_BIT: u8 = 1 << 6;
pub const FLAG_H_BIT: u8 = 1 << 5;
pub const FLAG_C_BIT: u8 = 1 << 4;

pub enum Flag {
    Z(bool),
    N(bool),
    H(bool),
    C(bool),
}

#[derive(PartialEq, Clone)]
pub struct CPU {
    pub register_a: u8,
    pub register_f: u8,
    pub register_b: u8,
    pub register_c: u8,
    pub register_d: u8,
    pub register_e: u8,
    pub register_h: u8,
    pub register_l: u8,
    pub register_sp: u16,
    pub register_pc: u16,
    interrupt_master_enable_flag: bool,
}

macro_rules! set_register_value {
    ($self:expr, $reg:expr, $($variant:ident => $field:ident),*) => {
        match $reg {
            $(
                RegisterValue::$variant(value) => {
                    $self.$field = *value;
                }
            )*
            _ => {}
        }
    };

    ($self:expr, $reg:expr, $($variant:ident => ($field1:ident, $field2:ident)),*) => {
        match $reg {
            $(
                RegisterValue::$variant(value) => {
                    $self.$field1 = (*value & 0xFF) as u8;
                    $self.$field2 = ((*value >> 8) & 0xFF) as u8;
                }
            )*
            _ => {}
        }
    };
}

impl Display for CPU {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Register\nAF: {:02X}{:02X} \nBC: {:02X}{:02X}\nDE: \
            {:02X}{:02X}\nHL: {:02X}{:02X}\nSP: {:04X}\nPC: {:04X}",
            self.register_a,
            self.register_f,
            self.register_b,
            self.register_c,
            self.register_d,
            self.register_e,
            self.register_h,
            self.register_l,
            self.register_sp,
            self.register_pc
        )
    }
}

macro_rules! ld {
    ($self:expr, $mem:ident, $variant:ident, $get_mem:ident, $len:expr) => {{
        let v = $self.$get_mem($mem);
        $self.set_value(&RegisterValue::$variant(v));
        $len
    }};
    ($self:expr, $mem:ident, "(HL)", $get_mem:ident, $len:expr) => {{
        let v = $self.$get_mem($mem);
        $self.set_mem_hl($mem, v);
        $len
    }};
    ($self:expr, $to_v:ident, $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            $self.set_value(&RegisterValue::$to_v(v));
        }
        $len
    }};
    ($self:expr, $mem:ident, ff($to_v:ident), $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            if let RegisterValue::$to_v(addr0) = $self.get_value(&RegisterValue::$to_v(0)) {
                let addr = addr0 as u16 + 0xff00;
                $mem.set(addr, v);
            }
        }
        $len
    }};
    ($self:expr, $mem:ident, ($to_v:ident), $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            if let RegisterValue::$to_v(addr) = $self.get_value(&RegisterValue::$to_v(0)) {
                $mem.set(addr, v);
            }
        }
        $len
    }};
    ($self:expr, $mem:ident, $to_v:ident, ($from_v:ident), $len:expr) => {{
        if let RegisterValue::$from_v(addr) = $self.get_value(&RegisterValue::$from_v(0)) {
            $self.set_value(&RegisterValue::$to_v($mem.get(addr)));
        }
        $len
    }};
    ($self:expr, $mem:ident, "(HL)", $mem_shift:tt, $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            if let RegisterValue::HL(addr) = $self.get_value(&RegisterValue::HL(0)) {
                $mem.set(addr, v);
                $self.set_value(&RegisterValue::HL(addr $mem_shift 1))
            }
        }
        $len
    }};
    ($self:expr, $mem:ident, $to_v:ident, "(HL)", $mem_shift:tt, $len:expr) => {{
        if let RegisterValue::HL(addr) = $self.get_value(&RegisterValue::HL(0)) {
            let v = $mem.get(addr);
            $self.set_value(&RegisterValue::HL(addr $mem_shift 1));
            $self.set_value(&RegisterValue::$to_v(v));
        }
        $len
    }};
    ($self:expr, $mem:ident, "(a8)", $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            let addr = $self.get_mem_a8($mem);
            $mem.set(addr, v);
        }
        $len
    }};
    ($self:expr, $mem:ident, "(a16)", $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            let addr = $self.get_mem_u16($mem);
            $mem.set(addr, v);
        }
        $len
    }};
}
macro_rules! ldh {
    ($self:expr, $mem:ident, $to_v:ident, "(a8)", $len:expr) => {{
        let addr = $self.get_mem_u8($mem) as u16 + 0xff00;
        let v = $mem.get(addr);
        $self.set_value(&RegisterValue::$to_v(v));
        $len
    }};
    ($self:expr, $mem:ident, $to_v:ident, "(a16)", $len:expr) => {{
        let addr = $self.get_mem_u16($mem);
        let v = $mem.get(addr);
        $self.set_value(&RegisterValue::$to_v(v));
        $len
    }};
    ($self:expr, $mem:ident, "(a8)", $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            let addr = $self.get_mem_u8($mem) as u16 + 0xff00;
            $mem.set(addr, v);
        }
        $len
    }};
}
macro_rules! push {
    ($self:expr, $mem:ident, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            $self.register_sp -= 1;
            $mem.set($self.register_sp, (v >> 8) as u8);
            $self.register_sp -= 1;
            $mem.set($self.register_sp, (v & 0xff) as u8);
        }
        $len
    }};
}

macro_rules! pop {
    ($self:expr, $mem:ident, AF, $len:expr) => {{
        $self.register_a = $mem.get($self.register_sp + 1);
        $self.register_f = ($mem.get($self.register_sp) & 0xf0);
        $self.register_sp += 2;
        $len
    }};
    ($self:expr, $mem:ident, $reg:ident, $len:expr) => {{
        let v =
            $mem.get($self.register_sp) as u16 + (($mem.get($self.register_sp + 1) as u16) << 8);
        $self.set_value(&RegisterValue::$reg(v));
        $self.register_sp += 2;
        $len
    }};
}

macro_rules! dec {
    ($self:expr, $mem:ident, (HL), $len:expr) => {{
        if let RegisterValue::HL(addr) = $self.get_value(&RegisterValue::HL(0)) {
            let v = $mem.get(addr);
            let h = ((v & 0xf) == 0);
            let v = v - 1;
            $mem.set(addr, v);
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(true));
            $self.set_flag(&Flag::H(h));
        }
        $len
    }};
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let h = ((v & 0xf) == 0);
            let v = v - 1;
            $self.set_value(&RegisterValue::$reg(v));
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(true));
            $self.set_flag(&Flag::H(h));
        }
        $len
    }};
    ($self:expr, $reg:ident, $len:expr, no_flag) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let v = v - 1;
            $self.set_value(&RegisterValue::$reg(v));
        }
        $len
    }};
}

macro_rules! inc {
    ($self:expr, $mem:ident, (HL), 12) => {{
        if let RegisterValue::HL(addr) = $self.get_value(&RegisterValue::HL(0)) {
            let v = $mem.get(addr);
            let v = v + 1;
            $mem.set(addr, v);
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H((v & 0xf) == 0));
        }
        12
    }};
    ($self:expr, $reg:ident, 4) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let v = v + 1;
            $self.set_value(&RegisterValue::$reg(v));
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H((v & 0xf) == 0));
        }
        4
    }};
    ($self:expr, $reg:ident, 8) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let v = v + 1;
            $self.set_value(&RegisterValue::$reg(v));
        }
        8
    }};
}

macro_rules! xor {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            $self.register_a ^= v;
            let z = $self.register_a == 0;
            $self.set_flag(&Flag::Z(z));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(false));
        }
        $len
    }};
}
macro_rules! or {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            $self.register_a |= v;
            let z = $self.register_a == 0;
            $self.set_flag(&Flag::Z(z));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(false));
        }
        $len
    }};
}
macro_rules! and {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            $self.register_a &= v;
            $self.set_flag(&Flag::Z($self.register_a == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(true));
            $self.set_flag(&Flag::C(false));
        }
        $len
    }};
    ($self:expr, $mem:ident, "d8", $len:expr) => {{
        $self.register_a &= $self.get_mem_u8($mem);
        let z = $self.register_a == 0;
        $self.set_flag(&Flag::Z(z));
        $self.set_flag(&Flag::N(false));
        $self.set_flag(&Flag::H(true));
        $self.set_flag(&Flag::C(false));
        $len
    }};
}
macro_rules! sub {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let z = ($self.register_a == v);
            let h = ($self.register_a & 0xf) < (v & 0xf);
            let c = $self.register_a < v;
            $self.register_a -= v;
            $self.set_flag(&Flag::Z(z));
            $self.set_flag(&Flag::N(true));
            $self.set_flag(&Flag::H(h));
            $self.set_flag(&Flag::C(c));
        }
        $len
    }};
}
macro_rules! add {
    ($self:expr, $reg_to:ident, $reg_from:ident, 4) => {{
        if let RegisterValue::$reg_to(v0) = $self.get_value(&RegisterValue::$reg_to(0)) {
            if let RegisterValue::$reg_from(v1) = $self.get_value(&RegisterValue::$reg_from(0)) {
                let h = (v1 & 0xf) > (0xf - (v0 & 0xf));
                let c = v1 > (0xff - v0);
                let z = (v0 + v1 == 0);
                $self.set_value(&RegisterValue::$reg_to(v0 + v1));
                $self.set_flag(&Flag::Z(z));
                $self.set_flag(&Flag::N(false));
                $self.set_flag(&Flag::H(h));
                $self.set_flag(&Flag::C(c));
            }
        }
        4
    }};
    ($self:expr, $reg_to:ident, $reg_from:ident, 8) => {{
        if let RegisterValue::$reg_to(v0) = $self.get_value(&RegisterValue::$reg_to(0)) {
            if let RegisterValue::$reg_from(v1) = $self.get_value(&RegisterValue::$reg_from(0)) {
                let h = (v1 & 0xfff) > (0xfff - (v0 & 0xfff));
                let c = v1 > (0xffff - v0);
                $self.set_value(&RegisterValue::$reg_to(v0 + v1));
                $self.set_flag(&Flag::N(false));
                $self.set_flag(&Flag::H(h));
                $self.set_flag(&Flag::C(c));
            }
        }
        8
    }};
    ($self:expr, $mem:ident, $reg:ident, (HL), $len:expr) => {{
        if let RegisterValue::$reg(v0) = $self.get_value(&RegisterValue::$reg(0)) {
            let v1 = $self.get_mem_hl($mem);

            let h = (v1 & 0xf) > (0xf - (v0 & 0xf));
            let c = v1 > (0xff - v0);
            let z = (v0 + v1 == 0);
            $self.set_value(&RegisterValue::$reg(v0 + v1));
            $self.set_flag(&Flag::Z(z));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(h));
            $self.set_flag(&Flag::C(c));
        }
        $len
    }};
}
macro_rules! bit {
    ($self:expr, $reg:ident, $shift:expr, $len:expr) => {{
        let v = ($self.$reg) & (1 << $shift);
        $self.set_flag(&Flag::Z(v == 0));
        $self.set_flag(&Flag::N(false));
        $self.set_flag(&Flag::H(true));
        $len
    }};
}
macro_rules! check_condition {
    ($self:expr, $flag:ident) => {{
        $self.get_flag(&Flag::$flag(false))
    }};
    ($self:expr, "N", $flag:ident) => {{
        !$self.get_flag(&Flag::$flag(false))
    }};
}

macro_rules! call {
    ($self:expr, $mem:ident, "a16", $len:expr) => {{
        let v = $self.get_mem_u16($mem);
        $self.register_sp -= 1;
        $mem.set($self.register_sp, ($self.register_pc >> 8) as u8);
        $self.register_sp -= 1;
        $mem.set($self.register_sp, ($self.register_pc & 0xff) as u8);
        $self.register_pc = v;
        $len
    }};
    ($self:expr, $mem:ident, $flag:ident, "a16", $len0:expr, $len1:expr) => {{
        let v = $self.get_mem_u16($mem);
        let c = check_condition!($self, $flag);
        if c {
            $self.register_sp -= 1;
            $mem.set($self.register_sp, ($self.register_pc >> 8) as u8);
            $self.register_sp -= 1;
            $mem.set($self.register_sp, ($self.register_pc & 0xff) as u8);
            $self.register_pc = v;
            $len0
        } else {
            $len1
        }
    }};
}

macro_rules! rl {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let c = (v >= 128);
            let mut v = v << 1;
            if $self.get_flag(&Flag::C(false)) {
                v += 1;
            }
            $self.set_value(&RegisterValue::$reg(v));
            // set flag
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(c));
        }
        $len
    }};
}

macro_rules! jr {
    ($self:expr, $mem:ident, $len:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        $self.register_pc = ($self.register_pc as i16 + v) as u16;
        $len
    }};
    ($self:expr, $mem:ident, $flag:ident, $len0:expr, $len1:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        let c = check_condition!($self, $flag);
        if c {
            $self.register_pc = ($self.register_pc as i16 + v) as u16;
            $len0
        } else {
            $len1
        }
    }};
    ($self:expr, $mem:ident, "N", $flag:ident, $len0:expr, $len1:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        let c = check_condition!($self, "N", $flag);
        if c {
            $self.register_pc = ($self.register_pc as i16 + v) as u16;
            $len0
        } else {
            $len1
        }
    }};
}

macro_rules! jp {
    ($self:expr, $mem:ident, (HL), $len:expr) => {{
        $self.register_pc = (($self.register_h as u16) << 8) + $self.register_l as u16;
        $len
    }};
    ($self:expr, $mem:ident, "a16", $len:expr) => {{
        let addr = $self.get_mem_u16($mem);
        $self.register_pc = addr;
        $len
    }};
    ($self:expr, $mem:ident, $flag:ident, "a16", $len0:expr, $len1:expr) => {{
        let c = check_condition!($self, $flag);
        let addr = $self.get_mem_u16($mem);
        if c {
            $self.register_pc = addr;
            $len0
        } else {
            $len1
        }
    }};
}

fn rla(cpu: &mut CPU, len: u8) -> u8 {
    let c = cpu.register_a >= 128;
    let mut v = cpu.register_a << 1;
    if cpu.get_flag(&Flag::C(false)) {
        v += 1;
    }
    cpu.register_a = v;
    cpu.set_flag(&Flag::Z(false));
    cpu.set_flag(&Flag::N(false));
    cpu.set_flag(&Flag::H(false));
    cpu.set_flag(&Flag::C(c));
    len
}

fn ret(cpu: &mut CPU, mem: &memory::Memory, len: u8) -> u8 {
    let v = mem.get(cpu.register_sp) as u16 + ((mem.get(cpu.register_sp + 1) as u16) << 8);
    cpu.register_sp += 2;
    cpu.register_pc = v;
    len
}

macro_rules! ret {
    ($self:expr, $mem:ident, $flag:ident, $len0:expr, $len1:expr) => {{
        let c = check_condition!($self, $flag);
        if c {
            ret($self, $mem, $len0)
        } else {
            $len1
        }
    }};
    ($self:expr, $mem:ident, "N", $flag:ident, $len0:expr, $len1:expr) => {{
        let c = check_condition!($self, "N", $flag);
        if c {
            ret($self, $mem, $len0)
        } else {
            $len1
        }
    }};
}
fn cp_impl(cpu: &mut CPU, t: u8) {
    let z = cpu.register_a == t;
    let h = (cpu.register_a & 0xf) < (t & 0xf);
    let c = cpu.register_a < t;
    cpu.set_flag(&Flag::Z(z));
    cpu.set_flag(&Flag::N(true));
    cpu.set_flag(&Flag::H(h));
    cpu.set_flag(&Flag::C(c));
}

fn cpl(cpu: &mut CPU) -> u8 {
    cpu.register_a = !cpu.register_a;
    cpu.register_f = cpu.register_f | 0b01100000;
    4
}

fn reti(cpu: &mut CPU, mem: &mut memory::Memory) -> u8 {
    cpu.interrupt_master_enable_flag = true;
    ret(cpu, mem, 16)
}

macro_rules! cp {
    ($self:expr, $mem:ident, "d8", $len:expr) => {{
        let t = $self.get_mem_u8($mem);
        cp_impl($self, t);
        $len
    }};
    ($self:expr, $mem:ident, (HL), $len:expr) => {{
        let t = $self.get_mem_hl($mem);
        cp_impl($self, t);
        $len
    }};
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            cp_impl($self, v);
        }
        $len
    }};
}

macro_rules! swap {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let swap_v = ((v & 0xf) << 4) + (v >> 4);
            $self.set_value(&RegisterValue::$reg(swap_v));
            // set flag
            $self.set_flag(&Flag::Z(swap_v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(false));
        }
        $len
    }};
}

macro_rules! rst {
    ($self:expr, $mem:ident, $num:expr, $len:expr) => {{
        $mem.set($self.register_sp - 1, ($self.register_pc >> 8) as u8);
        $mem.set($self.register_sp - 2, ($self.register_pc & 0xff) as u8);
        $self.register_sp -= 2;
        $self.register_pc = $num;
        $len
    }};
}
macro_rules! res {
    ($self:expr, $num:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let t = v & !(1 << $num);
            $self.set_value(&RegisterValue::$reg(t));
        }
        $len
    }};
}

macro_rules! sla {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let t = (v << 1);
            $self.set_value(&RegisterValue::$reg(t));
            // set flag
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(v >= 0b10000000));
        }
        $len
    }};
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_f: 0,
            register_b: 0,
            register_c: 0,
            register_d: 0,
            register_e: 0,
            register_h: 0,
            register_l: 0,
            register_sp: 0,
            register_pc: 0,
            interrupt_master_enable_flag: false,
        }
    }
    pub fn set_flag(&mut self, flag: &Flag) {
        let (v, flag_bit) = match flag {
            Flag::Z(v) => (v, FLAG_Z_BIT),
            Flag::N(v) => (v, FLAG_N_BIT),
            Flag::H(v) => (v, FLAG_H_BIT),
            Flag::C(v) => (v, FLAG_C_BIT),
        };
        if *v {
            self.register_f = self.register_f | flag_bit;
        } else {
            self.register_f = self.register_f & (0xff ^ flag_bit);
        }
    }

    pub fn set_value(&mut self, reg: &RegisterValue) {
        set_register_value!(self, reg,
            AF => (register_f, register_a),
            BC => (register_c, register_b),
            DE => (register_e, register_d),
            HL => (register_l, register_h)
        );

        set_register_value!(self, reg,
            A => register_a,
            F => register_f,
            B => register_b,
            C => register_c,
            D => register_d,
            E => register_e,
            H => register_h,
            L => register_l,
            SP => register_sp,
            PC => register_pc
        );
    }
    pub fn get_value(&self, reg: &RegisterValue) -> RegisterValue {
        match reg {
            RegisterValue::A(_) => RegisterValue::A(self.register_a),
            RegisterValue::F(_) => RegisterValue::F(self.register_f),
            RegisterValue::B(_) => RegisterValue::B(self.register_b),
            RegisterValue::C(_) => RegisterValue::C(self.register_c),
            RegisterValue::D(_) => RegisterValue::D(self.register_d),
            RegisterValue::E(_) => RegisterValue::E(self.register_e),
            RegisterValue::H(_) => RegisterValue::H(self.register_h),
            RegisterValue::L(_) => RegisterValue::L(self.register_l),
            RegisterValue::SP(_) => RegisterValue::SP(self.register_sp),
            RegisterValue::PC(_) => RegisterValue::PC(self.register_pc),
            RegisterValue::AF(_) => {
                RegisterValue::AF(((self.register_a as u16) << 8) + self.register_f as u16)
            }
            RegisterValue::BC(_) => {
                RegisterValue::BC(((self.register_b as u16) << 8) + self.register_c as u16)
            }
            RegisterValue::DE(_) => {
                RegisterValue::DE(((self.register_d as u16) << 8) + self.register_e as u16)
            }
            RegisterValue::HL(_) => {
                RegisterValue::HL(((self.register_h as u16) << 8) + self.register_l as u16)
            }
        }
    }
    pub fn get_flag(&self, flag: &Flag) -> bool {
        match flag {
            Flag::Z(_) => return (self.register_f & 0b10000000) != 0,
            Flag::N(_) => return (self.register_f & 0b01000000) != 0,
            Flag::H(_) => return (self.register_f & 0b00100000) != 0,
            Flag::C(_) => return (self.register_f & 0b00010000) != 0,
        }
    }
    fn get_pc_and_move(&mut self) -> u16 {
        let v = self.register_pc;
        self.register_pc += 1;
        v
    }
    fn get_mem_u8(&mut self, mem: &memory::Memory) -> u8 {
        mem.get(self.get_pc_and_move())
    }
    fn get_mem_a8(&mut self, mem: &memory::Memory) -> u16 {
        self.get_mem_u8(mem) as u16 + 0xff00
    }

    fn get_mem_u16(&mut self, mem: &memory::Memory) -> u16 {
        let v0 = self.get_mem_u8(mem) as u16;
        let v1 = self.get_mem_u8(mem) as u16;
        (v1 << 8) + v0
    }

    fn get_mem_hl(&self, mem: &memory::Memory) -> u8 {
        mem.get(((self.register_h as u16) << 8) + self.register_l as u16)
    }
    fn set_mem_hl(&self, mem: &mut memory::Memory, v: u8) {
        mem.set(((self.register_h as u16) << 8) + self.register_l as u16, v);
    }

    fn check_interrupt(&mut self, mem: &mut memory::Memory) -> bool {
        if ((mem.get(INTERRUPT_FLAG) & mem.get(INTERRUPT_ENABLE)) & 0b11111) > 0 {
            for (bit, target_addr) in INTERRPUT_LIST.iter().enumerate() {
                if mem.get_bit(INTERRUPT_FLAG, bit as u8)
                    && mem.get_bit(INTERRUPT_ENABLE, bit as u8)
                {
                    let flag_bit = 1 << bit;
                    let v = mem.get(INTERRUPT_FLAG) ^ flag_bit;
                    mem.set(INTERRUPT_FLAG, v);
                    mem.set(self.register_sp - 1, (self.register_pc >> 8) as u8);
                    mem.set(self.register_sp - 2, (self.register_pc & 0xff) as u8);
                    self.register_sp -= 2;
                    self.register_pc = *target_addr;
                    self.interrupt_master_enable_flag = false;
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    /// return cpu cycle in 4 MHz
    pub fn tick(&mut self, mem: &mut memory::Memory, break_point: Option<u16>) -> u8 {
        // check interrupt first but execute after
        let need_interrupt = self.interrupt_master_enable_flag;

        let op_addr: u8 = mem.get(self.get_pc_and_move());
        // println!(
        //     "instruction {:02x} pc:{:04X} interrupt {:08b} {:08b} {}",
        //     op_addr,
        //     self.register_pc - 1,
        //     mem.get(INTERRUPT_ENABLE),
        //     mem.get(INTERRUPT_FLAG),
        //     self.interrupt_master_enable_flag
        // );
        let need_break = if let Some(bp) = break_point {
            self.register_pc - 1 == bp
        } else {
            false
        };
        if need_break {
            self.register_pc -= 1;
            println!("before {}", self);
            println!(
                "mem sp {:02X} {:02X}",
                mem.get(self.register_sp),
                mem.get(self.register_sp + 1)
            );
            self.register_pc += 1;
        }
        let cpu_cycle_in_16mhz = match op_addr {
            0xcb => {
                let cb_op_addr: u8 = mem.get(self.get_pc_and_move());
                println!("cb instruction {:02x}", cb_op_addr);
                match cb_op_addr {
                    0x11 => rl!(self, C, 8),
                    0x27 => sla!(self, A, 8),
                    0x37 => swap!(self, A, 8),
                    0x7c => bit!(self, register_h, 7, 8),
                    0x87 => res!(self, 0, A, 8),
                    _ => {
                        self.register_pc -= 2;
                        todo!("cb opcode 0x{:02X} \n{}", cb_op_addr, self)
                    }
                }
            }
            0x00 => 4,
            0x01 => ld!(self, mem, BC, get_mem_u16, 12),
            0x03 => inc!(self, BC, 8),
            0x04 => inc!(self, B, 4),
            0x05 => dec!(self, B, 4),
            0x06 => ld!(self, mem, B, get_mem_u8, 8),
            0x09 => add!(self, HL, BC, 8),
            0x0b => dec!(self, BC, 8, no_flag),
            0x0c => inc!(self, C, 4),
            0x0d => dec!(self, C, 4),
            0x0e => ld!(self, mem, C, get_mem_u8, 8),
            0x11 => ld!(self, mem, DE, get_mem_u16, 12),
            0x12 => ld!(self, mem, (DE), A, 8),
            0x13 => inc!(self, DE, 8),
            0x14 => inc!(self, D, 4),
            0x15 => dec!(self, D, 4),
            0x16 => ld!(self, mem, D, get_mem_u8, 8),
            0x17 => rla(self, 4),
            0x18 => jr!(self, mem, 12),
            0x19 => add!(self, HL, DE, 8),
            0x1a => ld!(self, mem, A, (DE), 8),
            0x1b => dec!(self, DE, 8, no_flag),
            0x1c => inc!(self, E, 4),
            0x1d => dec!(self, E, 4),
            0x1e => ld!(self, mem, E, get_mem_u8, 8),
            0x20 => jr!(self, mem, "N", Z, 12, 8),
            0x21 => ld!(self, mem, HL, get_mem_u16, 12),
            0x22 => ld!(self, mem, "(HL)", +, A, 8),
            0x23 => inc!(self, HL, 8),
            0x24 => inc!(self, H, 4),
            0x25 => dec!(self, H, 4),
            0x26 => ld!(self, mem, H, get_mem_u8, 8),
            0x28 => jr!(self, mem, Z, 12, 8),
            0x2a => ld!(self, mem, A, "(HL)", +, 8),
            0x2b => dec!(self, HL, 8, no_flag),
            0x2c => inc!(self, L, 4),
            0x2d => dec!(self, L, 4),
            0x2e => ld!(self, mem, L, get_mem_u8, 8),
            0x2f => cpl(self),
            0x30 => jr!(self, mem, "N", C, 12, 8),
            0x31 => ld!(self, mem, SP, get_mem_u16, 12),
            0x32 => ld!(self, mem, "(HL)", -, A, 8),
            0x33 => inc!(self, SP, 8),
            0x34 => inc!(self, mem, (HL), 12),
            0x35 => dec!(self, mem, (HL), 12),
            0x36 => ld!(self, mem, "(HL)", get_mem_u8, 12),
            0x38 => jr!(self, mem, C, 12, 8),
            0x3b => dec!(self, SP, 8, no_flag),
            0x3c => inc!(self, A, 4),
            0x3d => dec!(self, A, 4),
            0x3e => ld!(self, mem, A, get_mem_u8, 8),
            0x40 => ld!(self, B, B, 4),
            0x41 => ld!(self, B, C, 4),
            0x42 => ld!(self, B, D, 4),
            0x43 => ld!(self, B, E, 4),
            0x44 => ld!(self, B, H, 4),
            0x45 => ld!(self, B, L, 4),
            0x46 => ld!(self, mem, B, (HL), 8),
            0x47 => ld!(self, B, A, 4),
            0x4a => ld!(self, C, D, 4),
            0x4b => ld!(self, C, E, 4),
            0x4c => ld!(self, C, H, 4),
            0x4d => ld!(self, C, L, 4),
            0x4e => ld!(self, mem, C, (HL), 8),
            0x4f => ld!(self, C, A, 4),
            0x50 => ld!(self, D, B, 4),
            0x51 => ld!(self, D, C, 4),
            0x52 => ld!(self, D, D, 4),
            0x53 => ld!(self, D, E, 4),
            0x54 => ld!(self, D, H, 4),
            0x55 => ld!(self, D, L, 4),
            0x56 => ld!(self, mem, D, (HL), 8),
            0x57 => ld!(self, D, A, 4),
            0x5e => ld!(self, mem, E, (HL), 8),
            0x5f => ld!(self, E, A, 4),
            0x60 => ld!(self, H, B, 4),
            0x61 => ld!(self, H, C, 4),
            0x62 => ld!(self, H, D, 4),
            0x63 => ld!(self, H, E, 4),
            0x64 => ld!(self, H, H, 4),
            0x65 => ld!(self, H, L, 4),
            0x66 => ld!(self, mem, H, (HL), 8),
            0x67 => ld!(self, H, A, 4),
            0x68 => ld!(self, L, B, 4),
            0x69 => ld!(self, L, C, 4),
            0x6e => ld!(self, mem, L, (HL), 8),
            0x70 => ld!(self, mem, (HL), B, 8),
            0x71 => ld!(self, mem, (HL), C, 8),
            0x72 => ld!(self, mem, (HL), D, 8),
            0x73 => ld!(self, mem, (HL), E, 8),
            0x74 => ld!(self, mem, (HL), H, 8),
            0x75 => ld!(self, mem, (HL), L, 8),
            0x77 => ld!(self, mem, (HL), A, 8),
            0x78 => ld!(self, A, B, 4),
            0x79 => ld!(self, A, C, 4),
            0x7a => ld!(self, A, D, 4),
            0x7b => ld!(self, A, E, 4),
            0x7c => ld!(self, A, H, 4),
            0x7d => ld!(self, A, L, 4),
            0x7e => ld!(self, mem, A, (HL), 8),
            0x86 => add!(self, mem, A, (HL), 8),
            0x87 => add!(self, A, A, 4),
            0x90 => sub!(self, B, 4),
            0x91 => sub!(self, C, 4),
            0x92 => sub!(self, D, 4),
            0x93 => sub!(self, E, 4),
            0x94 => sub!(self, H, 4),
            0x95 => sub!(self, L, 4),
            0xa0 => and!(self, B, 4),
            0xa1 => and!(self, C, 4),
            0xa2 => and!(self, D, 4),
            0xa3 => and!(self, E, 4),
            0xa4 => and!(self, H, 4),
            0xa5 => and!(self, L, 4),
            0xa7 => and!(self, A, 4),
            0xa8 => xor!(self, B, 4),
            0xa9 => xor!(self, C, 4),
            0xaa => xor!(self, D, 4),
            0xab => xor!(self, E, 4),
            0xac => xor!(self, H, 4),
            0xad => xor!(self, L, 4),
            0xaf => xor!(self, A, 4),
            0xb0 => or!(self, B, 4),
            0xb1 => or!(self, C, 4),
            0xb2 => or!(self, D, 4),
            0xb3 => or!(self, E, 4),
            0xb4 => or!(self, H, 4),
            0xb5 => or!(self, L, 4),
            0xb7 => or!(self, A, 4),
            0xb9 => cp!(self, C, 4), // need to verify
            0xbe => cp!(self, mem, (HL), 8),
            0xc0 => ret!(self, mem, "N", Z, 20, 8),
            0xc1 => pop!(self, mem, BC, 12),
            0xc3 => jp!(self, mem, "a16", 16),
            0xc5 => push!(self, mem, BC, 16),
            0xc7 => rst!(self, mem, 0x00, 16),
            0xc8 => ret!(self, mem, Z, 20, 8),
            0xc9 => ret(self, mem, 16),
            0xca => jp!(self, mem, Z, "a16", 16, 12),
            0xcc => call!(self, mem, Z, "a16", 24, 12),
            0xcd => call!(self, mem, "a16", 24),
            0xcf => rst!(self, mem, 0x08, 16),
            0xd1 => pop!(self, mem, DE, 12),
            0xd5 => push!(self, mem, DE, 16),
            0xd7 => rst!(self, mem, 0x10, 16),
            0xd9 => reti(self, mem),
            0xdf => rst!(self, mem, 0x18, 16),
            0xe0 => ldh!(self, mem, "(a8)", A, 12),
            0xe1 => pop!(self, mem, HL, 12),
            0xe2 => ld!(self, mem, ff(C), A, 8),
            0xe5 => push!(self, mem, HL, 16),
            0xe6 => and!(self, mem, "d8", 8),
            0xe7 => rst!(self, mem, 0x20, 16),
            0xe9 => jp!(self, mem, (HL), 4),
            0xea => ld!(self, mem, "(a16)", A, 16),
            0xef => rst!(self, mem, 0x28, 16),
            0xf0 => ldh!(self, mem, A, "(a8)", 12),
            0xf1 => pop!(self, mem, AF, 12),
            0xf3 => {
                self.interrupt_master_enable_flag = false;
                4
            }
            0xf5 => push!(self, mem, AF, 16),
            0xf7 => rst!(self, mem, 0x30, 16),
            0xfa => ldh!(self, mem, A, "(a16)", 16), // it's actually ld
            0xfb => {
                self.interrupt_master_enable_flag = true;
                4
            }
            0xfe => cp!(self, mem, "d8", 8),
            0xff => rst!(self, mem, 0x38, 16),
            _ => {
                self.register_pc -= 1;
                println!("opcode 0x{:02X} \n{}", op_addr, self);
                panic!()
            }
        };
        if need_interrupt {
            self.check_interrupt(mem);
        }
        if need_break {
            println!("after {}", self);
            panic!();
        }
        cpu_cycle_in_16mhz / 4
    }
}
