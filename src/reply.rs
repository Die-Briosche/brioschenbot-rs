
pub enum Comparator {
    Equals,
    Contains,
    Undefined,
}

pub enum ReplyType {
    Text,
    GifRandom,
    Undefined,
}

pub struct Reply {
    pub trigger: String,
    pub comparator: Comparator,
    pub ignore_case: bool,
    pub reply: String,
    pub reply_type: ReplyType,
    pub reply_flag: bool,
}