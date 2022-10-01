#[derive(PartialEq, Eq)]
enum Format {
    Bold, Italic, Underline
}

// Convert IRC text formatting to the format Discord uses
pub fn irc_to_dc(msg: &str) -> String {
    let mut new_msg = String::new();

    let mut formats = vec![];
    let mut code = false;

    println!("{:?}", msg);

    let mut chars = msg.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => new_msg += "\\\\",
            '*' => new_msg += "\\*",
            '_' => new_msg += "\\_",
            '~' => new_msg += "\\~",
            '`' => {
                new_msg.push('`');
                code = !code;
            },
            // Bold text
            '\x02' => {
                if !code && !formats.contains(&Format::Bold) {
                    new_msg += "**";
                    formats.push(Format::Bold)
                }
            },
            // Italics
            '\x1d' => {
                if !code && !formats.contains(&Format::Italic) {
                    new_msg += "*";
                    formats.push(Format::Italic)
                }
            },
            // Underline
            '\x1f' => {
                if !code && !formats.contains(&Format::Underline) {
                    new_msg += "__";
                    formats.push(Format::Underline)
                }
            },
            // Reset formatting
            '\x0f' => {
                if !code {
                    while let Some(fmt) = formats.pop() {
                        match fmt {
                            Format::Bold => new_msg += "**",
                            Format::Italic => new_msg += "*",
                            Format::Underline => new_msg += "__",
                        }
                    }
                }
            },
            // Color text (not translated)
            '\x03' => {
                chars.next();
                chars.next();
                if chars.peek() == Some(&',') {
                    chars.next();
                    chars.next();
                    chars.next();
                }
            },
            // Invert colors (not translated)
            '\x16' => (),
            c => new_msg.push(c)
        }
    }
    if code {
        new_msg.push('`');
    }
    while let Some(fmt) = formats.pop() {
        match fmt {
            Format::Bold => new_msg += "**",
            Format::Italic => new_msg += "*",
            Format::Underline => new_msg += "__",
        }
    }
    println!("{:?}", new_msg);
    new_msg
}

pub fn dc_to_irc() {

}
