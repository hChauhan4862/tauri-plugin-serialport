use crate::error::Error;
use crate::state::{ReadData, SerialportInfo, SerialportState};
use serialport::{DataBits, FlowControl, Parity, StopBits, SerialPortType, UsbPortInfo};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;
use tauri::{command, AppHandle, Runtime, State, Window};
use serde::Serialize;

/// `get_worksheet` Get the file sheet instance according to `path` and `sheet_name`.
fn get_serialport<T, F: FnOnce(&mut SerialportInfo) -> Result<T, Error>>(
    state: State<'_, SerialportState>,
    path: String,
    f: F,
) -> Result<T, Error> {
    match state.serialports.lock() {
        Ok(mut map) => match map.get_mut(&path) {
            Some(serialport_info) => f(serialport_info),
            None => {
                Err(Error::String("Serial Port Not Found".to_string()))
            }
        },
        Err(error) =>  Err(Error::String(format!("Cannot get a file lock! {} ", error))),
    }
}

fn get_data_bits(value: Option<usize>) -> DataBits {
    match value {
        Some(value) => match value {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            8 => DataBits::Eight,
            _ => DataBits::Eight,
        },
        None => DataBits::Eight,
    }
}

fn get_flow_control(value: Option<String>) -> FlowControl {
    match value {
        Some(value) => match value.as_str() {
            "Software" => FlowControl::Software,
            "Hardware" => FlowControl::Hardware,
            _ => FlowControl::None,
        },
        None => FlowControl::None,
    }
}

fn get_parity(value: Option<String>) -> Parity {
    match value {
        Some(value) => match value.as_str() {
            "Odd" => Parity::Odd,
            "Even" => Parity::Even,
            _ => Parity::None,
        },
        None => Parity::None,
    }
}

fn get_stop_bits(value: Option<usize>) -> StopBits {
    match value {
        Some(value) => match value {
            1 => StopBits::One,
            2 => StopBits::Two,
            _ => StopBits::Two,
        },
        None => StopBits::Two,
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct SerialPortInfo {
    port_name: String,
    port_type: String,
    vid: Option<String>,
    pid: Option<String>,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
}

fn port_type_to_string(port_type: &SerialPortType) -> String {
    match port_type {
        SerialPortType::UsbPort(_) => "USB".to_string(),
        SerialPortType::PciPort => "PCI".to_string(),
        SerialPortType::BluetoothPort => "Bluetooth".to_string(),
        SerialPortType::Unknown => "Unknown".to_string(),
    }
}

fn port_info_to_serial_port_info(port_info: &UsbPortInfo, port_name: &str) -> SerialPortInfo {
    let vid = format!("{:04x}", port_info.vid);
    let pid = format!("{:04x}", port_info.pid);
    let default_manufacturer = "Unknown".to_string();
    let manufacturer = port_info.manufacturer.as_ref().unwrap_or(&default_manufacturer).to_owned();
    let product = port_info.product.as_ref().unwrap_or(&default_manufacturer).to_owned();
    let serial_number = port_info.serial_number.clone();
    SerialPortInfo {
        port_name: port_name.to_owned(),
        port_type: format!("USB"),
        vid: Some(vid),
        pid: Some(pid),
        manufacturer: Some(manufacturer),
        product: Some(product),
        serial_number,
    }
}

/// `available_ports` Get available serial ports
#[command]
pub fn available_ports() -> Vec<SerialPortInfo> {
    let mut list = match serialport::available_ports() {
        Ok(list) => list,
        Err(_) => vec![],
    };
    list.sort_by(|a, b| a.port_name.cmp(&b.port_name));
    
    println!("Available ports: {:?}", list)
        
    list.iter()
        .map(|port| {
            match &port.port_type {
                SerialPortType::UsbPort(info) => {
                    port_info_to_serial_port_info(&info, &port.port_name)
                },
                _ => SerialPortInfo {
                    port_name: port.port_name.clone(),
                    port_type: port_type_to_string(&port.port_type),
                    vid: None,
                    pid: None,
                    manufacturer: None,
                    product: None,
                    serial_number: None,
                },
            }
        })
        .collect()
}



/// `cacel_read` Cancel read data from serial port
#[command]
pub async fn cancel_read<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
) -> Result<(), Error> {
    get_serialport(state, path.clone(), |serialport_info| {
        match &serialport_info.sender {
            Some(sender) => match sender.send(1) {
                Ok(_) => {}
                Err(error) => {
                    return Err(Error::String(format!("Failed to cancel read: {}", error)));
                }
            },
            None => {}
        }
        serialport_info.sender = None;
        println!("Canceled read data from {}", &path);
        Ok(())
    })
}

/// `close` Close serial port
#[command]
pub fn close<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut serialports) => {
            if serialports.remove(&path).is_some() {
                Ok(())
            } else {
                print!("Port {} is not opened", path);
                Err(Error::String(format!("Port {} is not opened", path)))
            }
        }
        Err(error) => {
            println!("Cannot get lock: {}", error);
            Err(Error::String(format!("Cannot get lock: {}", error)))
        }
    }
}

/// `close_all` Close all serial ports
#[command]
pub fn close_all<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut map) => {
            for serialport_info in map.values() {
                if let Some(sender) = &serialport_info.sender {
                    match sender.send(1) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("Failed to cancel read: {}", error);
                            return Err(Error::String(format!("Failed to cancel read: {}", error)));
                        }
                    }
                }
            }
            map.clear();
            Ok(())
        }
        Err(error) => {
            Err(Error::String(format!("Cannot get lock: {}", error)))
        }
    }
}

/// `force_close` Force close serial port
#[command]
pub fn force_close<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut map) => {
            if let Some(serial) = map.get_mut(&path) {
                if let Some(sender) = &serial.sender {
                    match sender.send(1) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("Cancel read data failed: {}", error);
                            return Err(Error::String(format!("Cancel read data failed: {}", error)));
                        }
                    }
                }
                map.remove(&path);
                Ok(())
            } else {
                Ok(())
            }
        }
        Err(error) => {
            Err(Error::String(format!("Cannot get lock: {}", error)))
        }
    }
}

/// `open` Open serial port
#[command]
pub fn open<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SerialportState>,
    _window: Window<R>,
    path: String,
    baud_rate: u32,
    data_bits: Option<usize>,
    flow_control: Option<String>,
    parity: Option<String>,
    stop_bits: Option<usize>,
    timeout: Option<u64>,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut serialports) => {
            if serialports.contains_key(&path) {
                return Err(Error::String(format!("Port {} is already opened", path)));
            }
            match serialport::new(path.clone(), baud_rate)
                .data_bits(get_data_bits(data_bits))
                .flow_control(get_flow_control(flow_control))
                .parity(get_parity(parity))
                .stop_bits(get_stop_bits(stop_bits))
                .timeout(Duration::from_millis(timeout.unwrap_or(200)))
                .open()
            {
                Ok(serial) => {
                    let data = SerialportInfo {
                        serialport: serial,
                        sender: None,
                    };
                    serialports.insert(path, data);
                    Ok(())
                }
                Err(error) => Err(Error::String(format!(
                    "Failed to open port {}: {}",
                    path,
                    error.description
                ))),
            }
        }
        Err(error) => {
            Err(Error::String(format!("Cannot get lock: {}", error)))
        }
    }
}

/// `read` Read data from serial port
#[command]
pub fn read<R: Runtime>(
    _app: AppHandle<R>,
    window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
    timeout: Option<u64>,
    size: Option<usize>,
) -> Result<(), Error> {
    get_serialport(state.clone(), path.clone(), |serialport_info| {
        if serialport_info.sender.is_some() {
            println!("Port {} is already reading", path);
            Ok(())
        } else {
            println!("Start reading data from {}", path);
            match serialport_info.serialport.try_clone() {
                Ok(mut serial) => {
                    let read_event = format!("plugin-serialport-read-{}", &path);
                    let (tx, rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();
                    serialport_info.sender = Some(tx);
                    thread::spawn(move || loop {
                        match rx.try_recv() {
                            Ok(_) => {
                                println!("Stopped reading data from {}", path);
                                break;
                            }
                            Err(error) => match error {
                                TryRecvError::Disconnected => {
                                    println!("Port {} is disconnected", path);
                                    break;
                                }
                                TryRecvError::Empty => {}
                            },
                        }
                        let mut serial_buf: Vec<u8> = vec![0; size.unwrap_or(1024)];
                        match serial.read(serial_buf.as_mut_slice()) {
                            Ok(size) => {
                                println!("Port {} read {} bytes", path, size);
                                match window.emit(
                                    &read_event,
                                    ReadData {
                                        data: &serial_buf[..size],
                                        size,
                                    },
                                ) {
                                    Ok(_) => {}
                                    Err(error) => {
                                        println!("Failed to emit event: {}", error);
                                    }
                                }
                            }
                            Err(_err) => {
                                println!("Port {} read failed", path);
                            }
                        }
                        thread::sleep(Duration::from_millis(timeout.unwrap_or(200)));
                    });
                }
                Err(error) => {
                    return Err(Error::String(format!("Failed to read port {}: {}", path, error)));
                }
            }
            Ok(())
        }
    })
}

/// `write` Write data to serial port
#[command]
pub fn write<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
    value: String,
) -> Result<usize, Error> {
    get_serialport(state, path.clone(), |serialport_info| {
        match serialport_info.serialport.write(value.as_bytes()) {
            Ok(size) => {
                Ok(size)
        }
            Err(error) => {
                Err(Error::String(format!(
                    "Failed to write data to port {}: {}",
                    &path, error
                )))
            }
        }
    })
}

/// `write` Write binary data to serial port
#[command]
pub fn write_binary<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
    value: Vec<u8>,
) -> Result<usize, Error> {
    get_serialport(state, path.clone(), |serialport_info| match serialport_info
        .serialport
        .write(&value)
    {
        Ok(size) => {
            Ok(size)
        }
        Err(error) => {
            Err(Error::String(format!(
                "Failed to write data to port {}: {}",
                &path, error
            )))
        }
    })
}
