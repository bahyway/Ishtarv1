//! HeptaScript Notebook — pure Rust logic (no web_sys).
//! DOM rendering is done in lib.rs.

pub const PRESETS: &[(&str, &str)] = &[
    ("All Particles", "SELECT all"),
    ("Tribe 0x0001", "SELECT all WHERE tribe_id = 0x0001"),
    ("Active Only", "SELECT all WHERE status = \"Active\""),
    ("Red Channel >150", "SELECT all WHERE Color_ID.R > 150"),
    ("Late Epochs", "SELECT latest WHERE epoch > 2"),
    ("Drifted Color", "SELECT all WHERE Color_ID.drift > 0.5"),
];

pub struct DemoParticle {
    pub name: &'static str,
    pub tribe_id: u16,
    pub epoch: u32,
    pub rgb: [u8; 3],
    pub status: &'static str,
    pub tags: &'static [&'static str],
}

pub const CORPUS: &[DemoParticle] = &[
    DemoParticle {
        name: "Ali_Karim",
        tribe_id: 0x0001,
        epoch: 1,
        rgb: [80, 200, 255],
        status: "Active",
        tags: &["identity", "verified"],
    },
    DemoParticle {
        name: "Fatima_Hassan",
        tribe_id: 0x0001,
        epoch: 2,
        rgb: [120, 200, 100],
        status: "Active",
        tags: &["identity", "verified"],
    },
    DemoParticle {
        name: "Omar_Said",
        tribe_id: 0x0002,
        epoch: 1,
        rgb: [200, 80, 80],
        status: "Critical",
        tags: &["alert", "drift"],
    },
    DemoParticle {
        name: "Nour_Ibrahim",
        tribe_id: 0x0001,
        epoch: 3,
        rgb: [180, 180, 50],
        status: "Watch",
        tags: &["diagnosis"],
    },
    DemoParticle {
        name: "Zahra_Ali",
        tribe_id: 0x0002,
        epoch: 2,
        rgb: [80, 200, 255],
        status: "Active",
        tags: &["identity"],
    },
    DemoParticle {
        name: "Khalid_Nasser",
        tribe_id: 0x0001,
        epoch: 1,
        rgb: [40, 40, 200],
        status: "Archive",
        tags: &["cold"],
    },
    DemoParticle {
        name: "Sara_Younis",
        tribe_id: 0x0003,
        epoch: 1,
        rgb: [200, 200, 200],
        status: "Pending",
        tags: &["staging"],
    },
    DemoParticle {
        name: "Ahmed_Fadel",
        tribe_id: 0x0001,
        epoch: 4,
        rgb: [255, 140, 0],
        status: "Warning",
        tags: &["diagnosis", "drift"],
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokKind {
    Keyword,
    Field,
    Operator,
    HexLit,
    NumLit,
    StrLit,
    Punct,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokKind,
    pub value: String,
}

pub fn tokenize(src: &str) -> Vec<Token> {
    const KW: &[&str] = &[
        "SELECT", "WHERE", "ALL", "LATEST", "HISTORY", "AND", "OR", "NOT", "PROBE", "LIMIT",
        "ORDER", "BY", "DESC", "ASC",
    ];
    const FD: &[&str] = &[
        "tribe_id",
        "Color_ID",
        "epoch",
        "kaki",
        "name",
        "status",
        "tags",
        "Color_ID.R",
        "Color_ID.G",
        "Color_ID.B",
        "Color_ID.drift",
    ];

    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '"' || c == '\'' {
            chars.next();
            let mut val = String::new();
            while let Some(&x) = chars.peek() {
                chars.next();
                if x == c {
                    break;
                }
                val.push(x);
            }
            tokens.push(Token {
                kind: TokKind::StrLit,
                value: format!("\"{val}\""),
            });
            continue;
        }
        if c == '0' && chars.clone().nth(1) == Some('x') {
            chars.next();
            chars.next();
            let mut val = "0x".to_string();
            while let Some(&x) = chars.peek() {
                if x.is_ascii_hexdigit() {
                    val.push(x);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token {
                kind: TokKind::HexLit,
                value: val,
            });
            continue;
        }
        if c.is_ascii_digit() {
            let mut val = String::new();
            while let Some(&x) = chars.peek() {
                if x.is_ascii_digit() || x == '.' {
                    val.push(x);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token {
                kind: TokKind::NumLit,
                value: val,
            });
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let mut val = String::new();
            while let Some(&x) = chars.peek() {
                if x.is_alphanumeric() || x == '_' || x == '.' {
                    val.push(x);
                    chars.next();
                } else {
                    break;
                }
            }
            let up = val.to_uppercase();
            let kind = if KW.contains(&up.as_str()) {
                TokKind::Keyword
            } else if FD.contains(&val.as_str()) {
                TokKind::Field
            } else {
                TokKind::Unknown
            };
            tokens.push(Token {
                kind,
                value: if kind == TokKind::Keyword { up } else { val },
            });
            continue;
        }
        if "=!<>".contains(c) {
            let mut op = String::from(c);
            chars.next();
            if let Some(&nx) = chars.peek() {
                if "=<>".contains(nx) {
                    op.push(nx);
                    chars.next();
                }
            }
            tokens.push(Token {
                kind: TokKind::Operator,
                value: op,
            });
            continue;
        }
        if "(),;".contains(c) {
            tokens.push(Token {
                kind: TokKind::Punct,
                value: c.to_string(),
            });
            chars.next();
            continue;
        }
        tokens.push(Token {
            kind: TokKind::Unknown,
            value: c.to_string(),
        });
        chars.next();
    }
    tokens
}

pub fn run_query(src: &str) -> Vec<&'static DemoParticle> {
    let up = src.to_uppercase();
    if up.contains("HISTORY") {
        return CORPUS.iter().take(1).collect();
    }
    if up.contains("LATEST") || up.contains("EPOCH > 2") {
        return CORPUS.iter().filter(|p| p.epoch > 2).collect();
    }
    if up.contains("COLOR_ID.DRIFT") {
        return CORPUS
            .iter()
            .filter(|p| {
                let [r, g, b] = p.rgb;
                (r as i32 - g as i32).abs() > 80 || (r as i32 - b as i32).abs() > 80
            })
            .collect();
    }
    if up.contains("0X0001") {
        return CORPUS.iter().filter(|p| p.tribe_id == 0x0001).collect();
    }
    if up.contains("0X0002") {
        return CORPUS.iter().filter(|p| p.tribe_id == 0x0002).collect();
    }
    if up.contains("STATUS") && up.contains("ACTIVE") {
        return CORPUS.iter().filter(|p| p.status == "Active").collect();
    }
    if up.contains("COLOR_ID.R") && up.contains("150") {
        return CORPUS.iter().filter(|p| p.rgb[0] > 150).collect();
    }
    CORPUS.iter().collect()
}

pub fn tok_color_css(k: &TokKind) -> &'static str {
    match k {
        TokKind::Keyword => "#bd93f9",
        TokKind::Field => "#8be9fd",
        TokKind::Operator => "#ffb86c",
        TokKind::HexLit => "#50fa7b",
        TokKind::NumLit => "#50fa7b",
        TokKind::StrLit => "#f1fa8c",
        TokKind::Punct => "#cdd6f4",
        TokKind::Unknown => "#ff5555",
    }
}

pub fn status_color_css(s: &str) -> &'static str {
    match s {
        "Active" => "#50fa7b",
        "Critical" => "#ff5555",
        "Warning" => "#ffb86c",
        "Watch" => "#f1fa8c",
        "Pending" => "#b0b0c0",
        "Archive" => "#6c7086",
        _ => "#cdd6f4",
    }
}

pub struct NotebookState {
    pub query: String,
}

impl NotebookState {
    pub fn new() -> Self {
        NotebookState {
            query: PRESETS[0].1.to_string(),
        }
    }
}

impl Default for NotebookState {
    fn default() -> Self {
        Self::new()
    }
}

impl NotebookState {
    pub fn set_query(&mut self, q: &str) {
        self.query = q.to_string();
    }
    pub fn tokens(&self) -> Vec<Token> {
        tokenize(&self.query)
    }
    pub fn results(&self) -> Vec<&'static DemoParticle> {
        run_query(&self.query)
    }
}
