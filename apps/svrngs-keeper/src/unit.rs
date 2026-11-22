//! A native card as a systemd user unit: the text svrngs-keeper writes, and
//! the OS keeps (svrn-docs KEEP_SHARING_BUILD.md §2). The settings are the
//! spike's, proved through five no-login boots (local-hosting-spike/recipe/
//! lhspike-model.service): a crash starts it again after 2 s, at most 4 times
//! in 60 s; the SIGTERM a stop sends is a stop, not a failure; and it starts
//! with the user's manager, which linger starts at boot.
//!
//! Pure: a value in, text out. Every word of the card is quoted and escaped
//! here, so nothing in it reads as a specifier, a variable or another word.

use crate::card::{Card, Native};
use crate::runtime::Name;
use crate::systemd::escape_specifiers;

/// The unit a kept card runs as: `svrngs-keeper-<name>.service`.
pub fn unit_name(name: &Name) -> String {
    format!("svrngs-keeper-{}.service", name.as_str())
}

/// The unit file for `card`, kept under `name`.
pub fn native(name: &Name, card: &Card, native: &Native) -> String {
    let name = name.as_str();
    let mut words: Vec<String> = native.command.iter().map(|w| word(w)).collect();
    // `@`: the word after the program is the first argument it is given,
    // the name it was started under (systemd.service(5), ExecStart=).
    if let Some(argv0) = &native.argv0 {
        words[0] = format!("@{}", words[0]);
        words.insert(1, word(argv0));
    }
    let exec = words.join(" ");
    let mut text = format!(
        "# svrngs-keeper keeps {name} from its card, version {version}.\n\
         # svrngs-keeper keep writes this file again: change the card, not this file.\n\
         [Unit]\n\
         Description={name}, kept by svrngs-keeper\n\
         StartLimitIntervalSec=60\n\
         StartLimitBurst=4\n\
         \n\
         [Service]\n\
         WorkingDirectory={folder}\n\
         ExecStart={exec}\n",
        version = card.version,
        folder = folder(&native.folder),
    );
    if !native.environment.is_empty() {
        text += &format!("PassEnvironment={}\n", native.environment.join(" "));
    }
    text += "Restart=on-failure\n\
             RestartSec=2\n\
             # A program that dies of the SIGTERM a stop sends may exit 143: a stop.\n\
             SuccessExitStatus=143\n\
             \n\
             [Install]\n\
             WantedBy=default.target\n";
    text
}

/// One word of `ExecStart=`, in double quotes: `%` doubled (a specifier),
/// `$` doubled (a variable), `\` and `"` escaped. A leading `~/` is `%h/`,
/// the home of whoever keeps it, as a shell would have read it.
fn word(w: &str) -> String {
    let (home, rest) = match w.strip_prefix("~/") {
        Some(rest) => ("%h/", rest),
        None => ("", w),
    };
    let rest = escape_specifiers(rest)
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "$$");
    format!("\"{home}{rest}\"")
}

/// `WorkingDirectory=` takes the rest of its line as one path, with
/// specifiers: `~` is systemd's own word for the home folder.
fn folder(f: &str) -> String {
    match f.strip_prefix("~/") {
        Some(rest) => format!("%h/{}", escape_specifiers(rest)),
        None => escape_specifiers(f),
    }
}
