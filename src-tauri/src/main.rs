// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};
use tauri::State;
use std::time::Duration;
use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

pub const LIBUSB_RECIPIENT_DEVICE: u8 = 0x00;

pub const LIBUSB_REQUEST_TYPE_VENDOR: u8 = 0x02 << 5;

#[derive(Debug)]
pub struct ConnectionState {
    serial_numbers: HashMap<u16, String>, // Maps addresses to serial numbers
    connected: Option<ConnectedDevice>
}

impl ConnectionState {
    fn new() -> ConnectionState {
        ConnectionState {
            serial_numbers: HashMap::new(),
            connected: None
        }
    }
}

#[derive(Debug)]
pub struct ConnectedDevice {
    device_handle: DeviceHandle<rusb::Context>,
    configuration_interface: Option<ConfigurationInterface>
}


#[derive(Debug)]
struct ConfigurationInterface {
    interface: u8,
    input: u8,
    output: u8
}

fn find_configuration_endpoints<T: UsbContext>(
    device: &Device<T>
) -> Option<ConfigurationInterface> {
    let device_desc = match device.device_descriptor() {
        Ok(d) => d,
        Err(_) => return None
    };
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                if interface_desc.class_code() == 0xff
                {
                    let mut endpoints = ConfigurationInterface {interface: interface_desc.interface_number(), input: 0, output: 0};
                    let mut has_input = false;
                    let mut has_output = false;
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == Direction::In {
                            endpoints.input = endpoint_desc.address();
                            has_input = true;
                        }
                        if endpoint_desc.direction() == Direction::Out {
                            endpoints.output = endpoint_desc.address();
                            has_output = false;
                        }
                    }
                    if has_input && has_output {
                        return Some(endpoints)
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
fn reboot_bootloader(connection_state: State<Mutex<ConnectionState>>) -> bool {
    let connection = connection_state.lock().unwrap();
    match &connection.connected {
        Some(d) => {
            let buf : [u8;0] = [];
            let r = d.device_handle.write_control(LIBUSB_RECIPIENT_DEVICE | LIBUSB_REQUEST_TYPE_VENDOR, 0, 0x2e8a, 0, &buf, Duration::from_millis(100));
            println!("Reboot Device: {}", r.is_err());

            return true;
        },
        None => {
            println!("No connection");
            return false;
        }
    }
}

#[tauri::command]
fn open(serial_number: &str, connection_state: State<Mutex<ConnectionState>>) -> bool {
    let context = match rusb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e),
    };

    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return false,
    };

    let mut connection = connection_state.lock().unwrap();
    for device in devices.iter() {
        let address : u16 = ((device.bus_number() as u16) << 8) | (device.address() as u16);
        match connection.serial_numbers.get(&address) {
            Some(sn) => {
                if sn == serial_number {
                    match device.open() {
                        Ok(mut handle) => {
                            let configuration_interface = find_configuration_endpoints(&device);
                            match &configuration_interface {
                                Some(i) => { handle.claim_interface(i.interface).unwrap(); },
                                None => { println!("Could not detect a configuration interface"); }
                            }
                            connection.connected = Some(ConnectedDevice {device_handle: handle, configuration_interface: configuration_interface });
                            return true
                        },
                        Err(_) => return false
                    }
                }
            },
            None => continue
        }
    }
    return false;
}

#[tauri::command]
fn poll_devices(connection_state: State<Mutex<ConnectionState>>) -> String {
    let mut device_list = Vec::with_capacity(10);
    let mut known_devices : HashSet<u16> = connection_state.lock().unwrap().serial_numbers.keys().cloned().collect();

    let context = match rusb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e),
    };

    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return serde_json::to_string(&device_list).unwrap(),
    };

    for device in devices.iter() {
        let address : u16 = ((device.bus_number() as u16) << 8) | (device.address() as u16);
        if known_devices.contains(&address) {
            device_list.push(connection_state.lock().unwrap().serial_numbers[&address].clone());
            known_devices.remove(&address);
            continue
        }
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        // println!("Device {:#x}:{:#x} {:#x} {:#x} {:#x}", device_desc.vendor_id(), device_desc.product_id(), device_desc.class_code(), device.bus_number(), device.address());

        if device_desc.vendor_id() == 0x2e8a && device_desc.product_id() == 0xfedd {
            match device.open() {
                Ok(handle) => {
                    let serial_number_string_index = device_desc.serial_number_string_index().unwrap();
                    let serial_number = handle.read_string_descriptor_ascii(serial_number_string_index);
                    match serial_number {
                        Ok(sn) => {
                            connection_state.lock().unwrap().serial_numbers.insert(address, sn.clone());
                            device_list.push(sn);
                        },
                        Err(e) => {
                            println!("Get serial number failed {}", e);
                            continue
                        }
                    }
                },
                Err(e) => {
                    println!("Open failed {}", e);
                    continue
                }
            }
        }
    }

    // Handle unplugged devices
    for address in known_devices
    {
        connection_state.lock().unwrap().serial_numbers.remove(&address);
    }

    serde_json::to_string(&device_list).unwrap()
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(ConnectionState::new()))
        .invoke_handler(tauri::generate_handler![reboot_bootloader, poll_devices, open])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
