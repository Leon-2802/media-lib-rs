#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rating {
    pub entry_id: i64,
    pub rating: i8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    pub entry_id: i64,
    pub added_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHistory {
    pub id: i64,
    pub query: String,
    pub at: i64,
}
