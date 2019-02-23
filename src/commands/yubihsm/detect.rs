use abscissa::Callable;
use std::process;
use yubihsm::connector::usb::Devices;

/// The `yubihsm detect` subcommand
#[derive(Debug, Default, Options)]
pub struct DetectCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    pub verbose: bool,
}

impl Callable for DetectCommand {
    /// Detect all YubiHSM2 devices connected via USB
    fn call(&self) {
        let devices = Devices::detect(Default::default()).unwrap_or_else(|e| {
            status_err!("couldn't detect USB devices: {}", e);

            // TODO: handle exits via abscissa
            process::exit(1);
        });

        if devices.is_empty() {
            status_err!("no YubiHSM2 devices detected!");
            process::exit(1);
        }

        println!("Detected YubiHSM2 USB devices:");

        for device in devices.iter() {
            println!(
                "- Serial #{} (bus {})",
                device.serial_number.as_str(),
                device.bus_number(),
            );
        }
    }
}
