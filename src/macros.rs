
macro_rules! hr {
    ($hresult:expr) => {
        {
            // It's important to only evaluate this once.
            let hresult = $hresult;
            if !::winapi::shared::winerror::SUCCEEDED(hresult) {
                use ::std::io::Write;
                use ::termcolor::{
                    Color,
                    ColorChoice,
                    ColorSpec,
                    StandardStream,
                    WriteColor,
                };

                let mut stderr = StandardStream::stderr(ColorChoice::Always);

                let empty = ColorSpec::new();

                let mut code = ColorSpec::new();
                code.set_fg(Some(Color::Green));
                code.set_intense(true);
                let code = code;

                let mut focus = ColorSpec::new();
                focus.set_fg(Some(Color::Red));
                focus.set_intense(true);
                let focus = focus;

                write!(stderr, "{}:{}: ", file!(), line!());

                match ::error::win_error_msg(hresult) {
                    ""  => { write!(stderr, "0x{:x}", hresult); },
                    msg => {
                        stderr.set_color(&focus).unwrap();
                        write!(stderr, "{}", msg);
                        stderr.set_color(&empty).unwrap();
                        write!(stderr, " (0x{:x}) ", hresult);
                    },
                }
                write!(stderr, "from:\n    ");

                stderr.set_color(&code).unwrap();
                write!(stderr, "{}\n",
                       stringify!($hresult).replace("\n", "\n    "));
                stderr.set_color(&empty).unwrap();
                Err(hresult)
            } else {
                Ok(())
            }
        }
    }
}
