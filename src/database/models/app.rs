#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Favorites {
    pub id: i32,
    pub series_id: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SearchHistory {
    pub id: i32,
    pub query: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SeriesRating {
    pub id: i32,
    pub series_id: i32,
    pub rating: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PartRating {
    pub id: i32,
    pub part_id: i32,
    pub rating: i32,
}

// ItemRating? Manga pages vs episodes etc
