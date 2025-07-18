#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PostId {
    pub number: usize,
    pub slug: String,
}

impl std::fmt::Display for PostId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.slug == "" {
            write!(f, "{}", self.number)
        } else {
            write!(f, "{}-{}", self.number, self.slug)
        }
    }
}
impl std::str::FromStr for PostId {
    type Err = PostIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"(^[0-9]+)(?:-([a-z0-9\-\_]*))?$").unwrap();
        let Some(captures) = re.captures(s) else {
            return Err(PostIdError::InvalidFormat);
        };

        let number = match captures.get(1) {
            Some(m) => &s[m.start()..m.end()],
            None => return Err(PostIdError::InvalidFormat),
        };
        let slug = match captures.get(2) {
            Some(m) => &s[m.start()..m.end()],
            None => "",
        };

        let Ok(number) = number.parse::<usize>() else {
            return Err(PostIdError::ParseNumberFailed);
        };
        let slug = String::from(slug);

        Ok(Self { number, slug })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PostIdError {
    ParseNumberFailed,
    InvalidFormat,
    Missing,
    Unknown,
}
impl std::error::Error for PostIdError {}
impl std::fmt::Display for PostIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostIdError::ParseNumberFailed => write!(f, "parsing number failed"),
            PostIdError::InvalidFormat => write!(f, "invalid format (should be <number>[-<slug>])"),
            PostIdError::Missing => write!(f, "missing parameter"),
            Self::Unknown => write!(f, "unk"),
        }
    }
}
impl std::str::FromStr for PostIdError {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parsing number failed" => Ok(Self::ParseNumberFailed),
            "invalid format (should be <number>[-<slug>])" => Ok(Self::InvalidFormat),
            _ => Ok(Self::Unknown),
        }
    }
}
