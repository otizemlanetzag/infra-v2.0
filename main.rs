use std::collections::HashMap;

// הגדרת מצבי האפליקציה
#[derive(Debug, PartialEq, Clone)]
pub enum AppMode {
    Terminal,
    Gui { screen_name: String },
}

// הגדרת האסימונים (Tokens) של השפה
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Main(String), 
    Send { screen: String, text: String, x: i32, y: i32, font: String },
    Rem(String),
    GetLib(String),
    For,
    If,
    While,
    Def(String),
    Connect(u16),
    Sen { text: String, port: u16, target: String },
    Ev(String),
    ArrowRun(String),
    ArrowRec { port: u16, text: Option<String> },
    ArrowPress(String),
    ArrowTouch(String),
    ArrowOther,
    Px { x: i32, y: i32, rgb: String },
    Pxs { x1: i32, y1: i32, x2: i32, y2: i32, rgb: String },
}

// פונקציית עזר לחילוץ מחרוזות ובדיקת תקינות גרשיים
fn parse_string(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') {
        Ok(s[1..s.len() - 1].to_string())
    } else if s.starts_with('\'') && s.ends_with('\'') {
        Ok(s[1..s.len() - 1].to_string())
    } else if (s.starts_with('"') && s.ends_with('\'')) || (s.starts_with('\'') && s.ends_with('"')) {
        Err(format!("שגיאת סינטקס: גרשיים לא תואמים בטקסט: {}", s))
    } else {
        Ok(s.to_string())
    }
}

// הלקסר (Lexer) - מפרק שורה בודדת ל-Token
pub fn lex_line(line: &str) -> Result<Option<Token>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("//") {
        return Ok(None);
    }

    if line.starts_with("main=") {
        let value = line[5..].trim().to_string();
        return Ok(Some(Token::Main(value)));
    }

    if line.starts_with("send(") && line.ends_with(')') {
        let content = &line[5..line.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        if parts.len() == 4 {
            return Ok(Some(Token::Send {
                screen: parts[0].to_string(),
                text: parse_string(parts[1])?,
                x: parts[2].parse().unwrap_or(0),
                y: parts[3].parse().unwrap_or(0),
                font: parts[4].to_string(),
            }));
        }
    } else if line.starts_with("rem(") && line.ends_with(')') {
        let text = parse_string(&line[4..line.len() - 1])?;
        return Ok(Some(Token::Rem(text)));
    } else if line.starts_with("getlib ") {
        return Ok(Some(Token::GetLib(line[7..].trim().to_string())));
    } else if line == "for" {
        return Ok(Some(Token::For));
    } else if line == "if" {
        return Ok(Some(Token::If));
    } else if line == "while" {
        return Ok(Some(Token::While));
    } else if line.starts_with("def ") {
        return Ok(Some(Token::Def(line[4..].trim().to_string())));
    } else if line.starts_with("connect(") && line.ends_with(')') {
        let port = line[8..line.len() - 1].trim().parse().unwrap_or(0);
        return Ok(Some(Token::Connect(port)));
    } else if line.starts_with("sen(") && line.ends_with(')') {
        let content = &line[4..line.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            return Ok(Some(Token::Sen {
                text: parse_string(parts[0])?,
                port: parts[1].parse().unwrap_or(0),
                target: parts[2].to_string(),
            }));
        }
    } else if line.starts_with("ev(") && line.ends_with(')') {
        return Ok(Some(Token::Ev(line[3..line.len() - 1].trim().to_string())));
    } else if line.starts_with("|->") {
        let inner = line[3..].trim();
        if inner.starts_with("rec(") && inner.ends_with(')') {
            let content = &inner[4..inner.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            let port = parts[0].parse().unwrap_or(0);
            let text = if parts.len() > 1 { Some(parse_string(parts[1])?) } else { None };
            return Ok(Some(Token::ArrowRec { port, text }));
        } else if inner.starts_with("press(") && inner.ends_with(')') {
            return Ok(Some(Token::ArrowPress(inner[6..inner.len() - 1].to_string())));
        } else if inner.starts_with("touch(") && inner.ends_with(')') {
            return Ok(Some(Token::ArrowTouch(inner[6..inner.len() - 1].to_string())));
        } else {
            return Ok(Some(Token::ArrowRun(inner.to_string())));
        }
    } else if line.starts_with("'->") {
        return Ok(Some(Token::ArrowOther));
    } else if line.starts_with("px(") && line.ends_with(')') {
        let content = &line[3..line.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            return Ok(Some(Token::Px {
                x: parts[0].parse().unwrap_or(0),
                y: parts[1].parse().unwrap_or(0),
                rgb: parts[2].to_string(),
            }));
        }
    } else if line.starts_with("pxs(") && line.ends_with(')') {
        let content = &line[4..line.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        if parts.len() == 5 {
            return Ok(Some(Token::Pxs {
                x1: parts[0].parse().unwrap_or(0),
                y1: parts[1].parse().unwrap_or(0),
                x2: parts[2].parse().unwrap_or(0),
                y2: parts[3].parse().unwrap_or(0),
                rgb: parts[4].to_string(),
            }));
        }
    }

    Err(format!("פקודה לא מוכרת: {}", line))
}

// המפרש (Interpreter)
pub struct Interpreter {
    pub mode: Option<AppMode>,
    pub libraries: Vec<String>,
    pub connected_ports: Vec<u16>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            mode: None,
            libraries: Vec::new(),
            connected_ports: Vec::new(),
        }
    }

    pub fn execute(&mut self, token: Token) {
        if self.mode.is_none() {
            if let Token::Main(val) = token {
                if val == "terminal" {
                    self.mode = Some(AppMode::Terminal);
                    println!("[מערכת] מצב אפליקציה: Terminal (התעלמות מפונטים ופיקסלים)");
                } else {
                    self.mode = Some(AppMode::Gui { screen_name: val.clone() });
                    println!("[מערכת] מצב אפליקציה: GUI. שם המסך הראשי: '{}'", val);
                }
                return;
            } else if !matches!(token, Token::GetLib(_)) {
                println!("[שגיאה] הסקריפט חייב להתחיל עם הגדרת main=");
                return;
            }
        }

        match token {
            Token::Main(_) => println!("[אזהרה] הגדרת main הופיעה שוב באמצע הסקריפט, מתעלם."),
            Token::Send { screen, text, x, y, font } => {
                match &self.mode {
                    Some(AppMode::Terminal) => println!("{}", text), // הדפסה חלקה בטרמינל
                    Some(AppMode::Gui { .. }) => println!("[GUI][מסך: {}] מציג: '{}' ב ({},{}) עם פונט {}", screen, text, x, y, font),
                    None => {}
                }
            }
            Token::Px { .. } | Token::Pxs { .. } | Token::Rem(_) => {
                if let Some(AppMode::Terminal) = self.mode {
                    return; // התעלמות מוחלטת בטרמינל
                }
                match token {
                    Token::Px { x, y, rgb } => println!("[GUI] צובע פיקסל ב ({},{}) בצבע {}", x, y, rgb),
                    Token::Pxs { x1, y1, x2, y2, rgb } => println!("[GUI] צובע מלבן מ ({},{}) עד ({},{}) בצבע {}", x1, y1, x2, y2, rgb),
                    Token::Rem(text) => println!("[GUI] מוחק את מופעי הטקסט: '{}'", text),
                    _ => {}
                }
            }
            Token::GetLib(lib) => {
                self.libraries.push(lib.clone());
                println!("[מערכת] מייבא ספרייה: {}", lib);
            }
            Token::Connect(port) => {
                self.connected_ports.push(port);
                println!("[רשת] מתחבר לפורט: {}", port);
            }
            Token::Sen { text, port, target } => println!("[רשת] שולח '{}' אל {} דרך פורט {}", text, target, port),
            Token::Ev(event) => println!("[אירוע] ממתין ל: {}", event),
            Token::ArrowRun(code) => println!("[זרימה] מחכה לריצת קוד: {}", code),
            Token::ArrowRec { port, text } => match text {
                Some(t) => println!("[רשת] מחכה לטקסט '{}' בפורט {}", t, port),
                None => println!("[רשת] מחכה למידע בפורט {}", port),
            },
            Token::ArrowPress(element) => println!("[קלט] מחכה ללחיצה על: {}", element),
            Token::ArrowTouch(element) => println!("[קלט] מחכה לנגיעה ב: {}", element),
            Token::ArrowOther => println!("[זרימה] ברירת מחדל ('->)"),
            Token::For => println!("[לולאה] התחלת לולאת for"),
            Token::If => println!("[תנאי] התחלת תנאי if"),
            Token::While => println!("[לולאה] התחלת לולאת while"),
            Token::Def(name) => println!("[פונקציה] הגדרת בלוק: {}", name),
        }
    }
}
