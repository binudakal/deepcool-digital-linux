mod devices;
mod monitor;
mod utils;

use colored::*;
use devices::*;
use utils::{args::Args, status::*};
use hidapi::HidApi;
use std::process::exit;

fn main() {
    // Read args
    let args = Args::read();

    // Find device
    let api = HidApi::new().unwrap_or_else(|err| {
        error!(err);
        exit(1);
    });
    let mut product_id = 0;
    for device in api.device_list() {
        if device.vendor_id() == DEFAULT_VENDOR_ID {
            if args.pid == 0 || device.product_id() == args.pid {
                product_id = device.product_id();
                println!("Device found: {}", device.product_string().unwrap().bright_green());
                println!("-----");
                break;
            }
        } else if device.vendor_id() == CH510_VENDOR_ID && device.product_id() == CH510_PRODUCT_ID {
            if args.pid == 0 || device.product_id() == args.pid {
                product_id = device.product_id();
                println!("Device found: {}", "CH510-MESH-DIGITAL".bright_green());
                println!("-----");
                break;
            }
        }
    }
    if product_id == 0 {
        if args.pid > 0 {
            error!("No DeepCool device was found with the specified PID");
        } else {
            error!("No DeepCool device was found");
        }
        exit(1);
    }

    // Connect to device and send datastream
    match product_id {
        // AK Series
        1..=4 => {
            println!("Supported modes: {} [default: {}]", "auto temp usage".bold(), ak_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ak_device = ak_series::Display::new(&args.mode, args.fahrenheit, args.alarm);
            // Print current configuration
            print_device_status(
                if args.mode == Mode::Default { ak_series::DEFAULT_MODE } else { args.mode },
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: if args.fahrenheit {
                        ak_series::TEMP_LIMIT_F
                    } else {
                        ak_series::TEMP_LIMIT_C
                    },
                },
                ak_series::POLLING_RATE,
            );
            // Display loop
            ak_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // LS Series
        6 => {
            println!("Supported modes: {} [default: {}]", "auto temp power".bold(), ls_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ls_device = ls_series::Display::new(&args.mode, args.fahrenheit, args.alarm);
            // Print current configuration
            print_device_status(
                if args.mode == Mode::Default { ls_series::DEFAULT_MODE } else { args.mode },
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: if args.fahrenheit {
                        ls_series::TEMP_LIMIT_F
                    } else {
                        ls_series::TEMP_LIMIT_C
                    },
                },
                ls_series::POLLING_RATE,
            );
            // Display loop
            ls_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // AG Series
        8 => {
            println!("Supported modes: {} [default: {}]", "auto temp usage".bold(), ag_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ag_device = ag_series::Display::new(&args.mode, args.alarm);
            // Print current configuration & warnings
            print_device_status(
                if args.mode == Mode::Default { ag_series::DEFAULT_MODE } else { args.mode },
                TemperatureUnit::Celsius,
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: ag_series::TEMP_LIMIT_C,
                },
                ag_series::POLLING_RATE,
            );
            if args.fahrenheit {
                warning!("Displaying ˚F is not supported, value will be ignored");
            }
            // Display loop
            ag_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // LD Series
        10 => {
            println!("Supported modes: {}", "auto".bold());
            // Connect to device
            let ld_device = ld_series::Display::new(args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                ld_series::DEFAULT_MODE,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: AlarmState::Auto,
                    temp_limit: if args.fahrenheit {
                        ld_series::TEMP_LIMIT_F
                    } else {
                        ld_series::TEMP_LIMIT_C
                    },
                },
                ld_series::POLLING_RATE,
            );
            if args.mode != Mode::Default {
                warning!("Display mode cannot be changed, value will be ignored");
            }
            if args.alarm {
                warning!("The alarm is hard-coded in your device, value will be ignored");
            }
            // Display loop
            ld_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // CH Series & MORPHEUS
        5 | 7 | 21 => {
            println!("Supported modes: {} [default: {}]", "auto temp usage".bold(), ch_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ch_device = ch_series::Display::new(&args.mode, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                if args.mode == Mode::Default { ch_series::DEFAULT_MODE } else { args.mode },
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0 },
                ch_series::POLLING_RATE,
            );
            if args.alarm {
                warning!("Alarm is not supported, value will be ignored");
            }
            // Display loop
            ch_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // CH510
        CH510_PRODUCT_ID => {
            println!("Supported modes: {} [default: {}]", "cpu gpu".bold(), ch510::DEFAULT_MODE.symbol());
            // Connect to device
            let ch510 = ch510::Display::new(&args.mode, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                if args.mode == Mode::Default { ch510::DEFAULT_MODE } else { args.mode },
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0 },
                ch510::POLLING_RATE,
            );
            if args.alarm {
                warning!("Alarm is not supported, value will be ignored");
            }
            // Display loop
            ch510.run(&api, CH510_VENDOR_ID, product_id);
        }
        _ => {
            println!("Device not yet supported!");
            println!("\nPlease create an issue on GitHub providing your device name and the following information:");
            let device = api.open(DEFAULT_VENDOR_ID, product_id).unwrap_or_else(|_| device_error());
            let info = device.get_device_info().unwrap();
            println!("Vendor ID: {}", info.vendor_id().to_string().bright_cyan());
            println!("Device ID: {}", info.product_id().to_string().bright_cyan());
            println!("Vendor name: {}", info.manufacturer_string().unwrap().bright_cyan());
            println!("Device name: {}", info.product_string().unwrap().bright_cyan());
        }
    }
}
