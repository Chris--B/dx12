

macro_rules! check_hresult {
    ($hresult:expr, $function:expr) => {
        use std::io::Write;
        use termcolor::{
            ColorChoice,
            ColorSpec,
            StandardStream,
            WriteColor,
        };

        let empty = ColorSpec::new();
        let specs = ::error::get_color_spec_catalog();

        let hresult = $hresult;
        if !winerror::SUCCEEDED(hresult) {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);

            stderr.set_color(&specs.file).unwrap();
            write!(stderr, "{}", file!());

            stderr.set_color(&empty).unwrap();
            write!(stderr, ":");

            stderr.set_color(&specs.line).unwrap();
            write!(stderr, "{}", line!());

            stderr.set_color(&empty).unwrap();
            write!(stderr, " ");

            stderr.set_color(&specs.func).unwrap();
            write!(stderr, "{}", stringify!($function));

            stderr.set_color(&empty).unwrap();
            write!(stderr, " failed with ");

            stderr.set_color(&specs.windows_msg).unwrap();
            write!(stderr, "{}", ::error::win_error_msg(hresult));

            stderr.set_color(&empty).unwrap();
            write!(stderr, " (");
            stderr.set_color(&specs.hresult).unwrap();
            write!(stderr, "0x{:x}", hresult);
            stderr.set_color(&empty).unwrap();
            write!(stderr, ")");

            stderr.set_color(&empty).unwrap();
            write!(stderr, "\n");

            return Err(1).into();
        }
    }
}
