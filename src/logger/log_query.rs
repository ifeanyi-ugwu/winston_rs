use chrono::{DateTime, Utc};

pub struct LogQuery {
    pub from: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub start: Option<usize>,
    pub order: Order,
    pub levels: Vec<String>,
    pub fields: Vec<String>,
    pub search_term: Option<String>,
}

pub enum Order {
    Ascending,
    Descending,
}

impl LogQuery {
    pub fn new() -> Self {
        LogQuery {
            from: None,
            until: None,
            limit: None,
            start: None,
            order: Order::Descending,
            levels: Vec::new(),
            fields: Vec::new(),
            search_term: None,
        }
    }

    pub fn from(mut self, from: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self
    }

    pub fn until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn start(mut self, start: usize) -> Self {
        self.start = Some(start);
        self
    }

    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }

    pub fn levels<S: Into<String>>(mut self, levels: Vec<S>) -> Self {
        self.levels = levels.into_iter().map(Into::into).collect();
        self
    }

    pub fn fields<S: Into<String>>(mut self, fields: Vec<S>) -> Self {
        self.fields = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn search_term<S: Into<String>>(mut self, search_term: S) -> Self {
        self.search_term = Some(search_term.into());
        self
    }
}
