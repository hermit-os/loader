use qemu_serial::QemuSerial;
use xlnx_serial::XlnxSerial;

pub mod qemu_serial;
pub mod xlnx_serial;

pub enum SerialSuccess<T> {
    Success(T),
    ERetry
}

pub enum SerialPort {
    Qemu(QemuSerial),
    Xlnx(XlnxSerial)
}

pub trait SerialDriver {
    fn init(&mut self);
    fn set_baud(&self, baud_rate: u32);
    fn putc(&mut self, c: u8) -> SerialSuccess<u8>;
    fn getc(&self) -> SerialSuccess<u8>;
    fn putstr(&mut self, s: &[u8]);
    fn get_addr(&self) -> u32;
    fn wait_empty(&mut self);
}

impl SerialDriver for SerialPort {
    fn init(&mut self) {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.init(),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.init(),
        }
    }

    fn set_baud(&self, baud_rate: u32) {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.set_baud(baud_rate),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.set_baud(baud_rate),
        }
    }

    fn putc(&mut self, c: u8) -> SerialSuccess<u8> {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.putc(c),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.putc(c),
        }
    }

    fn getc(&self) -> SerialSuccess<u8> {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.getc(),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.getc(),
        }
    }

    fn putstr(&mut self, s: &[u8]) {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.putstr(s),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.putstr(s),
        }
    }

    fn get_addr(&self) -> u32 {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.get_addr(),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.get_addr(),
        }
    }

    fn wait_empty(&mut self) {
        match self {
            SerialPort::Qemu(qemu_serial) => qemu_serial.wait_empty(),
            SerialPort::Xlnx(xlnx_serial) => xlnx_serial.wait_empty(),
        }
    }
}