// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use byteorder::{LittleEndian, ReadBytesExt};
use commands::Command;
use commands::FactoryReset;
use commands::GetStoredConfiguration;
use commands::GetVersion;
use commands::SaveConfiguration;
use commands::SetConfiguration;
use commands::SetFilterConfiguration;
use commands::SetPcm3060Configuration;
use commands::SetPreprocessingConfiguration;
use model::Filter;
use model::Filters;
use model::StructureTypes;
use rusb::{Device, DeviceHandle, Direction, UsbContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::default::Default;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::str;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
// Window shadow support
use tauri::Manager;
use window_shadows::set_shadow;

// Logging
use log::{debug, error, info, warn};
extern crate simplelog;
use simplelog::*;
use std::fs;
use std::fs::File;
use tauri::PathResolver;

mod model;
mod commands;

pub const LIBUSB_RECIPIENT_DEVICE: u8 = 0x00;
pub const LIBUSB_REQUEST_TYPE_VENDOR: u8 = 0x02 << 5;
pub const USB_TIMEOUT: Duration = Duration::from_millis(250);
const MAX_CFG_LEN: usize = 512;

#[derive(Debug, Default)]
pub struct ConnectionState {
    serial_numbers: HashMap<u16, String>, // Maps addresses to serial numbers
    connected: Option<ConnectedDevice>,
    error: bool,
}

#[derive(Debug)]
pub struct ConnectedDevice {
    device_handle: DeviceHandle<rusb::Context>,
    configuration_interface: ConfigurationInterface,
}

#[derive(Debug)]
struct ConfigurationInterface {
    interface: u8,
    input: u8,
    output: u8,
}

#[derive(Serialize)]
struct PollDeviceStatus {
    error: bool,
    device_list: Vec<String>,
}

impl PollDeviceStatus {
    fn new() -> PollDeviceStatus {
        PollDeviceStatus {
            error: false,
            device_list: Vec::with_capacity(10),
        }
    }
}

fn find_configuration_endpoints<T: UsbContext>(
    device: &Device<T>,
) -> Option<ConfigurationInterface> {
    let device_desc = match device.device_descriptor() {
        Ok(d) => d,
        Err(_) => return None,
    };
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                if interface_desc.class_code() != 0xff {
                    continue;
                }

                let mut input = None;
                let mut output = None;
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::In {
                        input = Some(endpoint_desc.address());
                    }
                    if endpoint_desc.direction() == Direction::Out {
                        output = Some(endpoint_desc.address());
                    }
                }

                if let (Some(input_addr), Some(out_addr)) = (input, output) {
                    return Some(ConfigurationInterface {
                        interface: interface_desc.interface_number(),
                        input: input_addr,
                        output: out_addr,
                    });
                }
            }
        }
    }
    None
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Preprocessing {
    preamp: f32,
    postEQGain: f32,
    reverse_stereo: bool,
}

impl Preprocessing {
    fn new(preamp: f32, postEQGain: f32, reverse_stereo: bool) -> Self {
        Preprocessing {
            preamp: preamp.log10() * 20.0,
            postEQGain: postEQGain.log10() * 20.0,
            reverse_stereo,
        }
    }

    fn to_payload(&self) -> Vec<u8> {
        let mut preprocessing_payload: Vec<u8> = Vec::new();
        // TODO: -1.0 as the firmware adds 1, cleanup later. Consider storing this value without the subtraction
        // to eliminate a math op and make the code more grokable?
        //preprocessing_payload.extend_from_slice(&(f32::powf(10.0, cfg.preprocessing.preamp/20.0) - 1.0).to_le_bytes());
        preprocessing_payload
            .extend_from_slice(&(f32::powf(10.0, self.preamp / 20.0) - 1.0).to_le_bytes());

        /* Send the post-EQ gain value from the UI. */
        preprocessing_payload
            .extend_from_slice(&(f32::powf(10.0, self.postEQGain / 20.0) - 1.0).to_le_bytes());

        preprocessing_payload.push(self.reverse_stereo as u8);
        preprocessing_payload.extend_from_slice(&[0u8; 3]);
        preprocessing_payload
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Codec {
    oversampling: bool,
    phase: bool,
    rolloff: bool,
    de_emphasis: bool,
}

impl Codec {
    fn new(oversampling: bool, phase: bool, rolloff: bool, de_emphasis: bool) -> Self {
        Self {
            oversampling,
            phase,
            rolloff,
            de_emphasis,
        }
    }

    fn to_payload(&self) -> Vec<u8> {
        vec![
            self.oversampling as u8,
            self.phase as u8,
            self.rolloff as u8,
            self.de_emphasis as u8,
        ]
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Config {
    preprocessing: Preprocessing,
    filters: Filters,
    codec: Codec,
}

#[derive(Serialize, Deserialize)]
struct VersionInfo {
    current_version: u16,
    minimum_supported_version: u16,
    git_hash: String,
    pico_sdk_version: String,
}

impl VersionInfo {
    fn from_buf(buf: &[u8]) -> Result<Self, String> {
        let mut cur = Cursor::new(buf);
        let _result_type_val = cur.read_u16::<LittleEndian>().unwrap();
        let _result_length_val = cur.read_u16::<LittleEndian>().unwrap();

        let _version_tlv_type_val = cur.read_u16::<LittleEndian>().unwrap();
        let _version_tlv_length_val = cur.read_u16::<LittleEndian>().unwrap();

        let current_version = cur.read_u16::<LittleEndian>().unwrap();
        let minimum_supported_version = cur.read_u16::<LittleEndian>().unwrap();
        cur.consume(4);
        let mut str_buf: Vec<u8> = Vec::new();
        cur.read_until(0u8, &mut str_buf).unwrap();
        str_buf.pop();
        let git_hash = match str::from_utf8(&str_buf) {
            Ok(s) => s.to_string(),
            Err(e) => return Err(format!("Invalid UTF-8 sequence: {}", e)),
        };
        str_buf.clear();
        cur.read_until(0u8, &mut str_buf).unwrap();
        str_buf.pop();
        let pico_sdk_version = match str::from_utf8(&str_buf) {
            Ok(s) => s.to_string(),
            Err(e) => return Err(format!("Invalid UTF-8 sequence: {}", e)),
        };

        Ok(Self {
            current_version,
            minimum_supported_version,
            git_hash,
            pico_sdk_version,
        })
    }
}

fn send_cmd(
    connection_state: State<'_, Mutex<ConnectionState>>,
    cmd: impl Command,
) -> Result<[u8; MAX_CFG_LEN], String> {
    let mut buf = Vec::new();
    cmd.write_as_binary(&mut buf);
    let mut connection = connection_state.lock().unwrap();

    let device = match &connection.connected {
        Some(x) => x,
        None => {
            info!("The device is not connected.");
            return Err("Not connected".to_owned());
        }
    };

    let interface = &device.configuration_interface;

    //println!("Write {} bytes to {}", buf.len(), interface.output);
    match device
        .device_handle
        .write_bulk(interface.output, &buf, USB_TIMEOUT) {
        Ok(_) => (),
        Err(err) => {
            error!("Failed to write to the configuration interface: {}", err);
            connection.error = true;
            return Err("Failed to write to the configuration interface".to_owned());
        }
    }

    let mut result = [0; MAX_CFG_LEN];
    let mut read_length: u16 = 0;
    let mut length: u16 = 4;
    while read_length < length {
        match device
            .device_handle
            .read_bulk(interface.input, &mut result, USB_TIMEOUT)
        {
            Ok(len) => {
                //println!("Read {} {}/{}", len, read_length, length);
                if read_length < 4 && len >= 4 {
                    let length_bytes: [u8; 2] = result[2..4].try_into().unwrap();
                    length = u16::from_le_bytes(length_bytes);
                    //println!("Length: {}", length);
                    if usize::from(length) > MAX_CFG_LEN {
                        return Err(format!("Overflow reading from the config interface, got {} bytes, max size is {} bytes.", length, MAX_CFG_LEN));
                    }
                }
                read_length += len as u16;
            }
            Err(err) => {
                connection.error = true;
                return Err(format!("Error reading from the configuration inteface: {}", err));
            }
        }
    }
    Ok(result)
}

#[tauri::command]
fn write_config(
    config: Config,
    connection_state: State<'_, Mutex<ConnectionState>>,
) -> Result<bool, String> {
    let prep = SetPreprocessingConfiguration::new(&config.preprocessing);
    let filters = SetFilterConfiguration::new(&config.filters);
    let codec = SetPcm3060Configuration::new(&config.codec);
    let cmd = SetConfiguration::new(prep, filters, codec);
    send_cmd(connection_state, cmd).map(|_| true)
}

#[tauri::command]
fn save_config(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<bool, String> {
    send_cmd(connection_state, SaveConfiguration::new()).map(|_| true)
}

#[tauri::command]
fn load_config(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<Config, String> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(StructureTypes::GetStoredConfiguration as u16).to_le_bytes());
    buf.extend_from_slice(&(4u16).to_le_bytes());

    let binding = send_cmd(connection_state, GetStoredConfiguration::new());
    let cfg = match &binding {
        Ok(x) => x,
        Err(e) => {
            // TODO: Check for NOK
            return Err(format!("Error reading config: {}", e));
        }
    };

    let mut cur = Cursor::new(cfg);
    let _result_type_val = cur.read_u16::<LittleEndian>().unwrap();
    let result_length_val = cur.read_u16::<LittleEndian>().unwrap();
    let mut position = 4;
    let mut cfg = Config::default();
    while position < result_length_val {
        let type_val = cur.read_u16::<LittleEndian>().unwrap();
        let length_val = cur.read_u16::<LittleEndian>().unwrap();
        match type_val {
            x if x == StructureTypes::PreProcessingConfiguration as u16 => {
                // +1 to maintain compatability with old firmwares
                let preamp = cur.read_f32::<LittleEndian>().unwrap() + 1.0;
                let postEQGain = cur.read_f32::<LittleEndian>().unwrap() + 1.0;
                let reverse_stereo = cur.read_u8().unwrap() != 0;

                cfg.preprocessing = Preprocessing::new(preamp, postEQGain, reverse_stereo);
                let _ = cur.seek(SeekFrom::Current(3)); // reserved bytes
            }
            x if x == StructureTypes::FilterConfiguration as u16 => {
                let end = cur.position() + (length_val - 4) as u64;
                while cur.position() < end {
                    cfg.filters.add(Filter::from_bytes(&mut cur)?)
                }

                if cur.position() != end {
                    return Err("Read off the end of the filters TLV".to_owned());
                }
            }
            x if x == StructureTypes::Pcm3060Configuration as u16 => {
                let oversampling = cur.read_u8().unwrap() != 0;
                let phase = cur.read_u8().unwrap() != 0;
                let rolloff = cur.read_u8().unwrap() != 0;
                let de_emphasis = cur.read_u8().unwrap() != 0;
                cfg.codec = Codec::new(oversampling, phase, rolloff, de_emphasis);
            }
            _ => {
                warn!("Unsupported TLV type {}", type_val);
            }
        }
        //println!("\tT: {} L: {}", type_val, length_val);
        position += length_val;
        cur.set_position(position as u64);
    }
    Ok(cfg)
}

#[tauri::command]
fn factory_reset(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<bool, String> {
    send_cmd(connection_state, FactoryReset::new()).map(|_| true)
}

#[tauri::command]
fn reboot_bootloader(connection_state: State<Mutex<ConnectionState>>) -> Result<(), String> {
    let connection = connection_state.lock().unwrap();
    let device = match &connection.connected {
        Some(x) => x,
        None => return Err("No connection".to_owned()),
    };

    let r = device.device_handle.write_control(
        LIBUSB_RECIPIENT_DEVICE | LIBUSB_REQUEST_TYPE_VENDOR,
        0,
        0x2e8a,
        0,
        &[],
        USB_TIMEOUT,
    );
    info!("Reboot Device: {}", r.is_err());

    return Ok(());
}

#[tauri::command]
fn read_version_info(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<VersionInfo, String> {
    let v = send_cmd(connection_state, GetVersion::new())?;
    let version = VersionInfo::from_buf(&v)?;
    Ok(version)
}

#[tauri::command]
fn open(serial_number: &str, connection_state: State<Mutex<ConnectionState>>) -> Result<(), String> {
    let context = rusb::Context::new().expect("Can't create libusb::Context::new()");

    let devices = context.devices()
        .map_err(|e| format!("Device not found: {}", e))?;

    let mut connection = connection_state.lock().unwrap();
    connection.connected = None;
    for device in devices.iter() {
        let address: u16 = ((device.bus_number() as u16) << 8) | (device.address() as u16);
        match connection.serial_numbers.get(&address) {
            Some(sn) => {
                if sn == serial_number {
                    match device.open() {
                        Ok(mut handle) => {
                            let configuration_interface = find_configuration_endpoints(&device);
                            let interface = match configuration_interface {
                                Some(i) => {
                                    handle.claim_interface(i.interface).unwrap();
                                    i
                                }
                                None => {
                                    return Err("Could not detect a configuration interface".to_owned());
                                }
                            };
                            info!(
                                "Opened the device at address {}, with serial number {}",
                                address, sn
                            );
                            connection.connected = Some(ConnectedDevice {
                                device_handle: handle,
                                configuration_interface: interface,
                            });
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(format!("Could not open {}", e));
                        }
                    }
                }
            }
            None => continue,
        }
    }
    return Err("Unknown error".to_owned());
}

#[tauri::command]
fn poll_devices(connection_state: State<Mutex<ConnectionState>>) -> PollDeviceStatus {
    let mut status = PollDeviceStatus::new();
    let mut known_devices: HashSet<u16> = connection_state
        .lock()
        .unwrap()
        .serial_numbers
        .keys()
        .cloned()
        .collect();

    // Flag any error condition to the frontend. This will cause it to try and reconnect.
    status.error = connection_state.lock().unwrap().error;
    connection_state.lock().unwrap().error = false;

    let context = rusb::Context::new().expect("Can't create libusb::Context::new()");

    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => {
            status.error = true;
            return status;
        }
    };

    for device in devices.iter() {
        let address: u16 = ((device.bus_number() as u16) << 8) | (device.address() as u16);
        if known_devices.contains(&address) {
            status
                .device_list
                .push(connection_state.lock().unwrap().serial_numbers[&address].clone());
            known_devices.remove(&address);
            continue;
        }
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        // println!("Device {:#x}:{:#x} {:#x} {:#x} {:#x}", device_desc.vendor_id(), device_desc.product_id(), device_desc.class_code(), device.bus_number(), device.address());

        if device_desc.vendor_id() == 0x2e8a && device_desc.product_id() == 0xfedd {
            info!("New device found at address {}", address);
            let handle = match device.open() {
                Ok(x) => x,
                Err(e) => {
                    error!("Open failed {}", e);
                    continue;
                }
            };

            let serial_number_string_index =
                device_desc.serial_number_string_index().unwrap();
            let serial_number =
                handle.read_string_descriptor_ascii(serial_number_string_index);

            let sn = match serial_number {
                Ok(x) => x,
                Err(e) => {
                    error!("Get serial number failed {}", e);
                    continue;
                }
            };

            info!("Device {} has serial number {}", address, sn);
            connection_state
                .lock()
                .unwrap()
                .serial_numbers
                .insert(address, sn.clone());
            status.device_list.push(sn);
        }
    }

    // Handle unplugged devices
    for address in known_devices {
        info!("The device at address {} was disconnected", address);
        connection_state
            .lock()
            .unwrap()
            .serial_numbers
            .remove(&address);
    }

    status
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_log_dir_path = app.path_resolver().app_log_dir().unwrap();
            let logfile = app_log_dir_path.join("headphones_toolbox.log");
            let lastlog = app_log_dir_path.join("headphones_toolbox.log.1");
            std::fs::create_dir_all(app_log_dir_path).unwrap();
            let _ = fs::rename(logfile.as_path(), lastlog.as_path());

            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Warn,
                    simplelog::Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Info,
                    simplelog::Config::default(),
                    File::create(logfile).unwrap(),
                ),
            ])
            .unwrap();
            let window = app.get_window("main").unwrap();
            #[cfg(any(windows, target_os = "macos"))]
            set_shadow(&window, true).expect("Unsupported platform!");
            info!("Headphones Toolbox Started");
            Ok(())
        })
        .manage(Mutex::new(ConnectionState::default()))
        .invoke_handler(tauri::generate_handler![
            reboot_bootloader,
            poll_devices,
            open,
            write_config,
            save_config,
            factory_reset,
            load_config,
            read_version_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
