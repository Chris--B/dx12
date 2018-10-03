
#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub force_warp: bool,
    pub enable_debug: bool,
    pub feature_level: u32
}

impl Config {
    pub fn load() -> Config {
        let matches = get_arg_matches();
        Config {
            force_warp:    matches.is_present("force-warp"),
            enable_debug:  !matches.is_present("no-debug"),
            feature_level: matches.value_of("feature-level")
                                   .expect("No feature level specified?")
                                   // Clap verifies this:
                                   .parse::<Dx12FeatureLevel>().unwrap()
                                   .into(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dx12FeatureLevel {
    E11_0,
    E11_1,
    E12_0,
    E12_1,
}

#[derive(Debug)]
pub struct InvalidFeatureLevel;

impl ::std::str::FromStr for Dx12FeatureLevel {
    type Err = InvalidFeatureLevel;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "11" | "11.0" | "11_0" => Ok(Dx12FeatureLevel::E11_0),
                   "11.1" | "11_1" => Ok(Dx12FeatureLevel::E11_1),
            "12" | "12.0" | "12_0" => Ok(Dx12FeatureLevel::E12_0),
                   "12.1" | "12_1" => Ok(Dx12FeatureLevel::E12_1),
            _                      => Err(InvalidFeatureLevel),
        }
    }
}

impl Into<u32> for Dx12FeatureLevel {
    fn into(self) -> u32 {
        use winapi::um::d3dcommon;
        match self {
            Dx12FeatureLevel::E11_0 => d3dcommon::D3D_FEATURE_LEVEL_11_0,
            Dx12FeatureLevel::E11_1 => d3dcommon::D3D_FEATURE_LEVEL_11_1,
            Dx12FeatureLevel::E12_0 => d3dcommon::D3D_FEATURE_LEVEL_12_0,
            Dx12FeatureLevel::E12_1 => d3dcommon::D3D_FEATURE_LEVEL_12_1,
        }
    }
}

fn get_arg_matches<'a>() -> ::clap::ArgMatches<'a> {
    use clap::{App, AppSettings, Arg};
    App
        // Metadata
        ::new("Dx12 Demo")
        .about("Draw things with DX12")
        .setting(AppSettings::DisableVersion)
        // Does this work?
        // .setting(AppSettings::DeriveDisplayOrder)

        // Adapter selection
        .arg(Arg::with_name("force-warp")
                .help("Force using the warp adapter")
                .long("force-warp")
                .short("w")
                .required(false)
                .overrides_with("adapter"))
        // TODO: Some way to select an adapter

        // Debug options
        .arg(Arg::with_name("debug")
                .display_order(3000)
                .help("Enable the DX12 runtime debug layer")
                .long("debug-layer")
                .overrides_with("no-debug-layer"))
        .arg(Arg::with_name("no-debug")
                .display_order(3001)
                .help("Disable the DX12 runtime debug layer")
                .long("no-debug-layer")
                .overrides_with("debug-layer"))

        // I change this enough to just make it an option.
        .arg(Arg::with_name("feature-level")
                .help("Force using a specific feature level for CreateDevice")
                .long("feature-level")
                .possible_values(&["11", "11.0", "11_0",
                                         "11.1", "11_1",
                                   "12", "12.0", "12_0",
                                         "12.1", "12_1"])
                .default_value("11_0"))

        // End
        .get_matches()
}
