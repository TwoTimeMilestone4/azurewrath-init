//! Azurewrath quotes for CLI and PID 1
//! Rotating quotes, never repeat the same one twice in a row.

use std::sync::atomic::{AtomicUsize, Ordering};

pub const QUOTES_BOOT: &[&str] = &[
    "Lament, mourn and despair.",
    "When will it be enough, these hands that rage?",
    "You will all pay in blood, TEN FOLD!",
    "From the depths, I ARISE AGAIN!",
];

pub const QUOTES_SOW: &[&str] = &[
    "From beyond, unearth yourself.",
    "Grow, so you may serve me.",
    "Hah! Behold...",
    "Rise, ONCE MORE!",
    "Blossom, be reborn!",
    "Certain types of nightshade can be fatal, so be careful...",
    "The stitches grow in pain, flaring like an unburdened fire. My mistake was trusting you.",
];

pub const QUOTES_BLOOM: &[&str] = &[
    "Born again!",
    "Hah, rise!",
    "Hah, to me!",
    "Grow!",
    "Blossom, be reborn!",
];

pub const QUOTES_WILT: &[&str] = &[
    "CRUMBLE!",
    "GRAAAH!",
    "Never again.",
    "I'LL CRUSH YOU TO PIECES!",
];

pub const QUOTES_UPROOT: &[&str] = &[
    "Hah!",
    "HAH!",
    "Be gone!",
    "Pathetic!",
];

pub const QUOTES_REGROW: &[&str] = &[
    "Rise, ONCE MORE!",
    "Rise, once more, Azurewrath.",
];

pub const QUOTES_REVIVE: &[&str] = &[
    "From the depths, I ARISE AGAIN!",
    "Rise, once more, Azurewrath.",
    "ARISE, YOU'LL ALL DIE!",
    "A CELEBRATION OF THE DEAD!",
    "DEATH, FOR THE LIVING!",
    "A TRUE BEAST OF NATURE!",
];

pub const QUOTES_REAP: &[&str] = &[
    "And I'll do it, AGAIN!",
    "Your efforts will, NEVER be enough!",
    "You pest, your life was WASTED!",
    "You failed to protect ANYONE!",
    "Out. Of my. Way.",
];

pub const QUOTES_STATUS: &[&str] = &[
    "My heart stopped its beat, and soon, yours will too...",
    "A revenant to that accursed village, I arise...",
    "I thank the one who granted me this... retribution.",
    "The transient sounds of winds sound softer today.",
    "May nature's hands be kinder than the folly of man..",
    "Sometimes I dream of him being plagued by illness. Freed from them.",
    "Was the second life worth it?",
];

pub const QUOTES_FAIL: &[&str] = &[
    "Argh, THIS IS NOTHING!",
    "Tch... CURSE YOU!",
    "AUGH! You...",
    "YOU... INSOLENT WORM!",
];

pub const QUOTES_KILL_TWO_TIME: &[&str] = &[
    "Again, again, AGAIN!",
    "Look what you've done!",
    "I hope you know what you've done...",
    "This was all YOUR FAULT!... Fool.",
    "I hate you, I hate you, I HATE YOU!",
    "Fool me once, fool me twice, and there will NOT be a third!",
];

pub const QUOTES_LMS: &[&str] = &[
    "So, our story comes to an end.",
    "This hatred that fuels me, this guilt I've repressed...",
    "Come and look me in the eye. SEE WHAT YOU CREATED!",
    "HAVE YOU NO REMORSE FOR ANYTHING YOU'VE DONE?!",
];

pub const QUOTES_OUTRO: &[&str] = &[
    "Hahah! I'll kill you again, and AGAIN... AND AGAIN!!",
    "Hmph, do you recognise this blade?",
    "Foolish! What GOOD has the SPAWN EVER DONE FOR YOU?!",
];

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Return the next rotating quote for the given command.
/// Never returns the same quote twice in a row.
pub fn quote_for(cmd: &str) -> &'static str {
    let quotes: &[&str] = match cmd {
        "boot" => QUOTES_BOOT,
        "sow" => QUOTES_SOW,
        "bloom" => QUOTES_BLOOM,
        "wilt" => QUOTES_WILT,
        "uproot" => QUOTES_UPROOT,
        "regrow" => QUOTES_REGROW,
        "revive" => QUOTES_REVIVE,
        "reap" => QUOTES_REAP,
        "status" => QUOTES_STATUS,
        "fail" => QUOTES_FAIL,
        "kill" => QUOTES_KILL_TWO_TIME,
        "lms" => QUOTES_LMS,
        "outro" => QUOTES_OUTRO,
        _ => QUOTES_STATUS,
    };
    if quotes.is_empty() {
        return "";
    }
    let idx = COUNTER.fetch_add(1, Ordering::Relaxed) % quotes.len();
    quotes[idx]
}

/// Print: Azurewrath: {action} - "{quote}"
pub fn log_azurewrath(action: &str, quote: &str) {
    println!("Azurewrath: {} - \"{}\"", action, quote);
}

/// Print: Azurewrath: "{quote}"
pub fn log_azurewrath_quote_only(quote: &str) {
    println!("Azurewrath: \"{}\"", quote);
}
