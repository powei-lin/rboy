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
}
pub struct CPU {
    register_a: u8,
    register_f: u8,
    register_b: u8,
    register_c: u8,
    register_d: u8,
    register_e: u8,
    register_h: u8,
    register_l: u8,
    register_sp: u16,
    register_pc: u16,
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
        }
    }
    pub fn set_value<T>(&mut self, reg: &RegisterValue) {
        match reg {
            RegisterValue::A(value) => {
                self.register_a = *value;
            }
            RegisterValue::F(value) => {
                self.register_f = *value;
            }
            RegisterValue::B(value) => {
                self.register_b = *value;
            }
            RegisterValue::C(value) => {
                self.register_c = *value;
            }
            RegisterValue::D(value) => {
                self.register_d = *value;
            }
            RegisterValue::E(value) => {
                self.register_e = *value;
            }
            RegisterValue::H(value) => {
                self.register_h = *value;
            }
            RegisterValue::L(value) => {
                self.register_l = *value;
            }
            RegisterValue::SP(value) => {
                self.register_sp = *value;
            }
            RegisterValue::PC(value) => {
                self.register_pc = *value;
            }
            RegisterValue::AF(value) => {
                self.register_f = *value as u8;
                self.register_a = (*value >> 8) as u8;
            }
            RegisterValue::BC(value) => {
                self.register_c = *value as u8;
                self.register_b = (*value >> 8) as u8;
            }
            RegisterValue::DE(value) => {
                self.register_e = *value as u8;
                self.register_d = (*value >> 8) as u8;
            }
        }
    }
}
