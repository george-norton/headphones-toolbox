// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use byteorder::{LittleEndian, ReadBytesExt};
use rusb::{Device, DeviceHandle, Direction, UsbContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::default::Default;
use std::fmt::format;
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
                        output: out_addr
                    });
                }
            }
        }
    }
    None
}

#[repr(u16)]
#[allow(dead_code)]
enum StructureTypes {
    // Commands/Responses, these are container TLVs. The Value will be a set of TLV structures.
    OK = 0,      // Standard response when a command was successful
    NOK,         // Standard error response
    FlashHeader, // A special container for the config stored in flash. Hopefully there is some useful
    // metadata in here to allow us to migrate an old config to a new version.
    GetVersion, // Returns the current config version, and the minimum supported version so clients
    // can decide if they can talk to us or not.
    SetConfiguration, // Updates the active configuration with the supplied TLVs
    GetActiveConfiguration, // Retrieves the current active configuration TLVs from RAM
    GetStoredConfiguration, // Retrieves the current stored configuration TLVs from Flash
    SaveConfiguration, // Writes the active configuration to Flash
    FactoryReset,     // Invalidates the flash memory

    // Configuration structures, these are returned in the body of a command/response
    PreProcessingConfiguration = 0x200,
    FilterConfiguration,
    Pcm3060Configuration,

    // Status structures, these are returned in the body of a command/response but they are
    // not persisted as part of the configuration
    VersionStatus = 0x400,
}

enum FilterType {
    Lowpass = 0,
    Highpass,
    BandpassSkirt,
    BandpassPeak,
    Notch,
    Allpass,
    Peaking,
    LowShelf,
    HighShelf,
    CustomIIR,
}

#[derive(Serialize, Deserialize, Default)]
struct Filter {
    filter_type: String,
    q: f32,
    f0: f32,
    db_gain: f32,
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct Preprocessing {
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

    fn to_buf(&self) -> Vec<u8> {
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

#[derive(Serialize, Deserialize, Default)]
struct Codec {
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

    fn to_buf(&self) -> Vec<u8> {
        vec![
            self.oversampling as u8,
            self.phase as u8,
            self.rolloff as u8,
            self.de_emphasis as u8,
        ]
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    preprocessing: Preprocessing,
    filters: Vec<Filter>,
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
    buf: &[u8],
) -> Result<[u8; MAX_CFG_LEN], &'static str> {
    let mut connection = connection_state.lock().unwrap();

    let device = match &connection.connected {
        Some(x) => x,
        None => {
            info!("The device is not connected.");
            return Err("Not connected");
        }
    };

    let interface = &device.configuration_interface;

    //println!("Write {} bytes to {}", buf.len(), interface.output);
    match device
        .device_handle
        .write_bulk(interface.output, &buf, USB_TIMEOUT)
    {
        Ok(_len) => (),
        Err(err) => {
            error!("Failed to write to the configuration interface: {}", err);
            connection.error = true;
            return Err("Failed to write to the configuration interface");
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
                        error!("Overflow reading from the config interface, got {} bytes, max size is {} bytes.", length, MAX_CFG_LEN);
                        return Err("Overflow error");
                    }
                }
                read_length += len as u16;
            }
            Err(err) => {
                error!("Error reading from the configuration inteface: {}", err);
                connection.error = true;
                return Err("Read Error");
            }
        }
    }
    Ok(result)
}

#[tauri::command]
fn write_config(
    config: &str,
    connection_state: State<'_, Mutex<ConnectionState>>,
) -> Result<bool, ()> {
    let cfg = match serde_json::from_str::<Config>(config) {
        Ok(x) => x,
        Err(e) => {
            error!("Error serializing config: {}", e);
            info!("{}", config);
            return Ok(false);
        }
    };

    let mut filter_payload: Vec<u8> = Vec::new();
    for filter in cfg.filters.iter().filter(|f| f.enabled) {
        let filter_type_val: u8;
        let filter_args;

        match filter.filter_type.as_str() {
            "lowpass" => {
                filter_type_val = 0;
                filter_args = 2;
            }
            "highpass" => {
                filter_type_val = 1;
                filter_args = 2;
            }
            "bandpass_skirt" => {
                filter_type_val = 2;
                filter_args = 2;
            }
            "bandpass" | "bandpass_peak" => {
                filter_type_val = 3;
                filter_args = 2;
            }
            "notch" => {
                filter_type_val = 4;
                filter_args = 2;
            }
            "allpass" => {
                filter_type_val = 5;
                filter_args = 2;
            }
            "peaking" => {
                filter_type_val = 6;
                filter_args = 3;
            }
            "lowshelf" => {
                filter_type_val = 7;
                filter_args = 3;
            }
            "highshelf" => {
                filter_type_val = 8;
                filter_args = 3;
            }
            "custom_iir" => {
                filter_type_val = 9;
                filter_args = 6;
            }
            _ => return Ok(false),
        }
        filter_payload.push(filter_type_val);
        filter_payload.extend_from_slice(&[0u8; 3]);
        if filter_type_val == FilterType::CustomIIR as u8 {
            filter_payload.extend_from_slice(&filter.a0.to_le_bytes());
            filter_payload.extend_from_slice(&filter.a1.to_le_bytes());
            filter_payload.extend_from_slice(&filter.a2.to_le_bytes());
            filter_payload.extend_from_slice(&filter.b0.to_le_bytes());
            filter_payload.extend_from_slice(&filter.b1.to_le_bytes());
            filter_payload.extend_from_slice(&filter.b2.to_le_bytes());
        } else {
            filter_payload.extend_from_slice(&filter.f0.to_le_bytes());
            if filter_args == 3 {
                filter_payload.extend_from_slice(&filter.db_gain.to_le_bytes());
            }
            filter_payload.extend_from_slice(&filter.q.to_le_bytes());
        }
    }

    let preprocessing_payload: Vec<u8> = cfg.preprocessing.to_buf();
    let codec_payload = cfg.codec.to_buf();

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(StructureTypes::SetConfiguration as u16).to_le_bytes());
    buf.extend_from_slice(
        &((16 + filter_payload.len() + preprocessing_payload.len() + codec_payload.len()) as u16)
            .to_le_bytes(),
    );
    buf.extend_from_slice(&(StructureTypes::PreProcessingConfiguration as u16).to_le_bytes());
    buf.extend_from_slice(&((4 + preprocessing_payload.len()) as u16).to_le_bytes());
    buf.extend_from_slice(&preprocessing_payload);
    buf.extend_from_slice(&(StructureTypes::FilterConfiguration as u16).to_le_bytes());
    buf.extend_from_slice(&((4 + filter_payload.len()) as u16).to_le_bytes());
    buf.extend_from_slice(&filter_payload);
    buf.extend_from_slice(&(StructureTypes::Pcm3060Configuration as u16).to_le_bytes());
    buf.extend_from_slice(&((4 + codec_payload.len()) as u16).to_le_bytes());
    buf.extend_from_slice(&codec_payload);

    match &send_cmd(connection_state, &buf) {
        Ok(_) => return Ok(true), // TODO: Check for NOK
        Err(e) => {
            error!("Error writing config: {}", e);
            return Err(());
        }
    }
}

#[tauri::command]
fn save_config(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<bool, ()> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(StructureTypes::SaveConfiguration as u16).to_le_bytes());
    buf.extend_from_slice(&(4u16).to_le_bytes());

    match &send_cmd(connection_state, &buf) {
        Ok(_) => return Ok(true), // TODO: Check for NOK
        Err(e) => {
            error!("Error saving config: {}", e);
            return Err(());
        }
    }
}

#[tauri::command]
fn load_config(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<String, ()> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(StructureTypes::GetStoredConfiguration as u16).to_le_bytes());
    buf.extend_from_slice(&(4u16).to_le_bytes());

    let binding = send_cmd(connection_state, &buf);
    let cfg = match &binding {
        Ok(x) => x,
        Err(e) => {
            // TODO: Check for NOK
            error!("Error reading config: {}", e);
            return Err(());
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
                cur.seek(SeekFrom::Current(3)); // reserved bytes
            }
            x if x == StructureTypes::FilterConfiguration as u16 => {
                let end = cur.position() + (length_val - 4) as u64;
                while cur.position() < end {
                    let filter_type = cur.read_u8().unwrap();
                    cur.seek(SeekFrom::Current(3)); // reserved bytes
                    let filter_args;

                    let mut filter = Filter::default();
                    filter.enabled = true;
                    match filter_type {
                        x if x == FilterType::Lowpass as u8 => {
                            filter.filter_type = "lowpass".to_string();
                            filter_args = 2;
                        }
                        x if x == FilterType::Highpass as u8 => {
                            filter.filter_type = "highpass".to_string();
                            filter_args = 2;
                        }
                        x if x == FilterType::BandpassSkirt as u8 => {
                            filter.filter_type = "bandpass_skirt".to_string();
                            filter_args = 2;
                        }
                        x if x == FilterType::BandpassPeak as u8 => {
                            filter.filter_type = "bandpass_peak".to_string();
                            filter_args = 2;
                        }
                        x if x == FilterType::Notch as u8 => {
                            filter.filter_type = "notch".to_string();
                            filter_args = 2;
                        }
                        x if x == FilterType::Allpass as u8 => {
                            filter.filter_type = "allpass".to_string();
                            filter_args = 2;
                        }
                        x if x == FilterType::Peaking as u8 => {
                            filter.filter_type = "peaking".to_string();
                            filter_args = 3;
                        }
                        x if x == FilterType::LowShelf as u8 => {
                            filter.filter_type = "lowshelf".to_string();
                            filter_args = 3;
                        }
                        x if x == FilterType::HighShelf as u8 => {
                            filter.filter_type = "highshelf".to_string();
                            filter_args = 3;
                        }
                        x if x == FilterType::CustomIIR as u8 => {
                            filter.filter_type = "custom_iir".to_string();
                            filter_args = 6;
                        }
                        _ => {
                            return {
                                error!("Unknown filter type {}", filter_type);
                                Err(())
                            }
                        }
                    }

                    if filter_type == FilterType::CustomIIR as u8 {
                        filter.a0 = cur.read_f64::<LittleEndian>().unwrap();
                        filter.a1 = cur.read_f64::<LittleEndian>().unwrap();
                        filter.a2 = cur.read_f64::<LittleEndian>().unwrap();
                        filter.b0 = cur.read_f64::<LittleEndian>().unwrap();
                        filter.b1 = cur.read_f64::<LittleEndian>().unwrap();
                        filter.b2 = cur.read_f64::<LittleEndian>().unwrap();
                    } else {
                        filter.f0 = cur.read_f32::<LittleEndian>().unwrap();
                        filter.db_gain;
                        if filter_args == 3 {
                            filter.db_gain = cur.read_f32::<LittleEndian>().unwrap();
                        }
                        filter.q = cur.read_f32::<LittleEndian>().unwrap();
                    }
                    cfg.filters.push(filter)
                }

                if cur.position() != end {
                    error!("Read off the end of the filters TLV");
                    return Err(());
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
    Ok(serde_json::to_string(&cfg).unwrap())
}

#[tauri::command]
fn factory_reset(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<bool, ()> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(StructureTypes::FactoryReset as u16).to_le_bytes());
    buf.extend_from_slice(&(4u16).to_le_bytes());

    match &send_cmd(connection_state, &buf) {
        Ok(_r) => return Ok(true), // TODO: Check for NOK
        Err(e) => {
            error!("Factory reset error: {}", e);
            return Err(());
        }
    }
}

#[tauri::command]
fn reboot_bootloader(connection_state: State<Mutex<ConnectionState>>) -> bool {
    let connection = connection_state.lock().unwrap();
    match &connection.connected {
        Some(device) => {
            let buf: [u8; 0] = [];
            let r = device.device_handle.write_control(
                LIBUSB_RECIPIENT_DEVICE | LIBUSB_REQUEST_TYPE_VENDOR,
                0,
                0x2e8a,
                0,
                &buf,
                USB_TIMEOUT,
            );
            info!("Reboot Device: {}", r.is_err());

            return true;
        }
        None => {
            warn!("No connection");
            return false;
        }
    }
}

#[tauri::command]
fn read_version_info(connection_state: State<'_, Mutex<ConnectionState>>) -> Result<String, ()> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(StructureTypes::GetVersion as u16).to_le_bytes());
    buf.extend_from_slice(&(4u16).to_le_bytes());

    let binding = send_cmd(connection_state, &buf);
    let v = match &binding {
        Ok(x) => x,
        Err(e) => {
            error!("Error reading device version information: {}", e);
            return Err(());
        }
    };

    let version = VersionInfo::from_buf(v).unwrap();
    Ok(serde_json::to_string(&version).unwrap())
}

#[tauri::command]
fn open(serial_number: &str, connection_state: State<Mutex<ConnectionState>>) -> bool {
    let context = match rusb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e),
    };

    let devices = match context.devices() {
        Ok(d) => d,
        Err(e) => {
            error!("Device not found: {}", e);
            return false;
        }
    };

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
                                    println!("Could not detect a configuration interface");
                                    return false;
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
                            return true;
                        }
                        Err(e) => {
                            error!("Could not open {}", e);
                            return false;
                        }
                    }
                }
            }
            None => continue,
        }
    }
    return false;
}

#[tauri::command]
fn poll_devices(connection_state: State<Mutex<ConnectionState>>) -> String {
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

    let context = match rusb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e),
    };

    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => {
            status.error = true;
            return serde_json::to_string(&status).unwrap();
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
            match device.open() {
                Ok(handle) => {
                    let serial_number_string_index =
                        device_desc.serial_number_string_index().unwrap();
                    let serial_number =
                        handle.read_string_descriptor_ascii(serial_number_string_index);
                    match serial_number {
                        Ok(sn) => {
                            info!("Device {} has serial number {}", address, sn);
                            connection_state
                                .lock()
                                .unwrap()
                                .serial_numbers
                                .insert(address, sn.clone());
                            status.device_list.push(sn);
                        }
                        Err(e) => {
                            error!("Get serial number failed {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("Open failed {}", e);
                    continue;
                }
            }
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

    serde_json::to_string(&status).unwrap()
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_log_dir_path = app.path_resolver().app_log_dir().unwrap();
            let logfile = app_log_dir_path.join("headphones_toolbox.log");
            let lastlog = app_log_dir_path.join("headphones_toolbox.log.1");
            std::fs::create_dir_all(app_log_dir_path).unwrap();
            fs::rename(logfile.as_path(), lastlog.as_path());

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
