use clap::{Parser, ValueEnum, crate_authors, crate_version};
use copypasta_ext::{prelude::*, x11_fork::ClipboardContext};
use std::{
    char, env, fmt,
    io::{self, Write},
};

#[derive(Parser)]
#[command(
    name = "cb",
    author = crate_authors!(", "),
    version = crate_version!(),
)]
/// Conveniently copy characters to clipboard.
///
/// Convenient CLI to copy commonly used characters that are difficult to type to clipboard.
struct Cli {
    /// Character you wish to copy to clipboard.
    #[clap(value_enum, required = true)]
    ch: CharVal,
}

// We use a clap::ValueEnum to conveniently handle different user input.
// However, this is a positional argument, not an option.
// E.g., you type
//     cb minus
// Instead of
//     cb --minus
//
// See different patterns for such input:
//   github.com/jakewilliami/gl/blob/bd5c7618/src/main.rs#L82-L88
//   github.com/jakewilliami/totp-tool/blob/c3375c48/src/main.rs#L7-L29
#[derive(Clone, ValueEnum)]
enum CharVal {
    // Dashes
    EnDash = 0x2013,
    EmDash = 0x2014,

    // Operators
    Minus = 0x2212,
    Times = 0x00D7,
    Div = 0x00F7,

    // Equality
    Sim = 0x223C,
    Approx = 0x2248,
    Gte = 0x2265,
    Lte = 0x2264,

    // Set Theory
    In = 0x2208,
    Ni = 0x220B,
    Union = 0x222A,
    Intersection = 0x2229,
    Subset = 0x2282,
    SubsetEq = 0x2286,
    Supset = 0x2283,
    SupsetEq = 0x2287,

    // Long Arrows
    RightArrow = 0x27F6,
    MapsTo = 0x27FC,
    LeftArrow = 0x27F5,
    MapsFrom = 0x27FB,

    // Other
    Prime = 0x2032,
    PlusMinus = 0x00B1,
    Degree = 0x00B0,
    TradeMark = 0x2122,
}

// Stolen from:
//   github.com/jakewilliami/<Leroux>/blob/350724a2/src/dash.rs#L125-L130
pub struct Char(char);

impl From<CharVal> for Char {
    fn from(x: CharVal) -> Self {
        let c = char::from_u32(x as u32).unwrap();
        Self(c)
    }
}

impl Char {
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let ch = Char::from(cli.ch);

    // Copy the requested character to clipboard
    copy_to_clipboard(&ch.as_str());

    // Print the requested character to stdout
    let mut stdout = io::stdout();
    writeln!(stdout, "{ch}")?;
    stdout.flush()?; // Make sure it actually writes out
    Ok(())
}

// Stolen from:
//   github.com/jakewilliami/totp-tool/blob/c3375c48/src/main.rs#L126-L148
fn copy_to_clipboard(s: &str) {
    // Try set clipboard for WSL or SSH first, falling back to `clipboard` if unavailable
    let set_res = clipboard_anywhere::set_clipboard(s);
    let get_res = clipboard_anywhere::get_clipboard();

    // Possible errors:
    //   1. Something has gone wrong if we can neither set nor get the clipboard
    let clipboard_unresponsive = set_res.is_err() && get_res.is_err();
    //   2. If we are not using SSH, get_res should be okay
    let local_clipboard_get_err = env::var("SSH_CLIENT").is_err() && get_res.is_err();
    //   3. We might be able to get the result from clipboard but it could be empty
    let clipboard_not_populated = get_res.is_ok() && get_res.unwrap().is_empty();

    // Clipboard should be populated, but if any of the above edge cases are true,
    // then we need additional handling for possible errors or a final attempt
    // at setting the clipboard.
    if clipboard_unresponsive || local_clipboard_get_err || clipboard_not_populated {
        // If the clipboard is empty, then we failed to set the clipboard using
        // clipboard_anywhere; as such, let's try setting the clipboard using an
        // X11-aware clipboard manager
        let result = std::panic::catch_unwind(|| {
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(s.to_string())
                .expect("Failed to set contents of clipboard");
        });

        if result.is_err() {
            eprintln!("Warning: 2FA code could not be copied to clipboard");
        }
    }
}

/*
If you want a non-interactive way to check if a certain backslash string, you can
defer to Julia:

```julia-repl
julia> completion("in")
'∈': Unicode U+2208 (category Sm: Symbol, math)

julia> get_uint_repr_from_completion("in")
"0x2208"

julia> completion("supset")
'⊃': Unicode U+2283 (category Sm: Symbol, math)
```

Source code:

```julia
using REPL

function completion(s::String)
    bslash_s = string('\\', s)
    ok, ret = REPL.REPLCompletions.bslash_completions(bslash_s, length(bslash_s))
    options, range, should_complete = ret
    @assert isone(length(options))
    char_s = only(options).bslash
    return only(char_s)  # use `only` to extract char from completion
end

using InteractiveUtils

function get_uint_repr_from_completion(s::String)
    ch = completion(s)
    h = string(UInt16(ch), base=16)
    u = string("0x", uppercase(h))
    InteractiveUtils.clipboard(u)
    return u
end
```
*/
