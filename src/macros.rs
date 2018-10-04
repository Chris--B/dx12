
macro_rules! make_fmt {
    ($arg:expr) => (format_args!("{:#?}", $arg));
    ($first:expr, $($args:expr),+) => (
        format_args!("{:#?}, {:#?}", $first, make_fmt!($($args),+))
    )
}

macro_rules! check_hresult {
    ($hresult:expr, $function:expr) => (check_hresult!($hresult, $function, ""));
    ($hresult:expr, $function:expr, $($args:expr),+) => {
        {
            use ::std::io::Write;
            use ::termcolor::{
                ColorChoice,
                ColorSpec,
                StandardStream,
                WriteColor,
            };

            let hresult: i32 = $hresult;
            if !::winapi::shared::winerror::SUCCEEDED(hresult) {
                let empty = ColorSpec::new();
                let specs = ::error::get_color_spec_catalog();

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
                // Force evaluation of '$function'.
                // Part of why we pass it to the macro is to validate that it's
                // a legal symbol. This does that.
                let _pfn = $function as *const () as usize;
                write!(stderr, "{}({}): ",
                       stringify!($function),
                       make_fmt!($($args),+));

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

                // The upper WORD of this hresult identifies the category of
                // the error's source.
                Err(hresult as u32)
            } else {
                Ok(())
            }
        }
    }
}
