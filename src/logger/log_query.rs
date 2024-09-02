use chrono::{DateTime, Duration, Utc};

use crate::LogEntry;

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
            from: Some(Utc::now() - Duration::days(1)),
            until: Some(Utc::now()),
            limit: Some(50),
            start: Some(0),
            order: Order::Descending,
            fields: Vec::new(),
            levels: Vec::new(),
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

    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check level
        if !self.levels.is_empty() && !self.levels.contains(&entry.level) {
            //println!("failed at levels check");
            return false;
        }

        // Check timestamp
        if let Some(from) = self.from {
            if let Some(timestamp) = entry.timestamp() {
                if timestamp < from {
                    //println!("failed at from check");
                    return false;
                }
            } else {
                println!("failed at from check");
                return false;
            }
        }

        if let Some(until) = self.until {
            if let Some(timestamp) = entry.timestamp() {
                if timestamp > until {
                    //println!("failed at until check");
                    return false;
                }
            } else {
                //println!("failed at until check");
                return false;
            }
        }

        // Check search term in message
        if let Some(ref search_term) = self.search_term {
            if !entry.message.contains(search_term) {
                //println!("failed at search term check");
                return false;
            }
        }

        // Check fields in meta data
        for field in &self.fields {
            if !entry.meta.contains_key(field) {
                //println!("failed at field check");
                return false;
            }
        }

        true
    }

    pub fn sort(&self, entries: &mut Vec<LogEntry>) {
        match self.order {
            Order::Ascending => entries.sort_by(|a, b| a.timestamp().cmp(&b.timestamp())),
            Order::Descending => entries.sort_by(|a, b| b.timestamp().cmp(&a.timestamp())),
        }
    }
}
