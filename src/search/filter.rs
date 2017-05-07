// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! full text search boyer more for know, maybe indexed search later, maintain cache of history
//! what about case sensitivity here? --> need to convert myself, fuck!
//!
//!
//! tag,title search: Use sift3 on single words(split by whitespace)
//!
//! use rayon

use super::searchparam::{TextFilter, FilterOperator, DateFilter,SearchFilter};
use serde_json::Value;
use distance::sift3;
use chrono::{TimeZone, DateTime, UTC, Duration};

fn find_string(name: &str, input: &Value) -> Option<String> {
    match input {
        &Value::Object(ref props) => {
            let o = props.get(name);
            if let Some(value) = o {
                match value {
                    &Value::String(ref text) => return Some(text.clone()),
                    _ => return None
                }
            } else {
                let results: Vec<Option<String>> = props.values().filter(|v| v.is_object()).map(|v| find_string(name, v)).filter(|o| o.is_some()).collect();
                if !results.is_empty() {
                    return results[0].clone();
                }
            }
            None
        }
        _ => {
            None
        }
    }
}

const MAX_SIFT_DISTANCE: f32 = 5f32;

pub fn fuzzy_search(needle: &str, haystack: &str) -> f32 {
    let lowercase_needle = needle.to_lowercase();
    let lowercase_haystack = haystack.to_lowercase();
    if lowercase_haystack.contains(lowercase_needle.as_str()) {
        return 0f32;
    }

    let words = lowercase_haystack.split_whitespace();
    let mut max = 100f32;
    for word in words {
        let distance = sift3(lowercase_needle.as_str(), word);
        if distance < max {
            max = distance;
        }
    }
    max
}

pub fn fuzzy_contains(needle: &str, haystack: &str) -> bool {
    fuzzy_search(needle, haystack) < MAX_SIFT_DISTANCE
}

pub fn filter_text(filter: &TextFilter, value: &Value) -> bool {
    let o = find_string(filter.field.as_str(), value);
    if let Some(value) = o {
        let value = value.to_lowercase();
        let expected = filter.text.clone().unwrap_or(String::new()).to_lowercase();
        match filter.operator {
            FilterOperator::Equal => return expected == value,
            FilterOperator::NotEqual => return expected != value,
            FilterOperator::FuzzyContains => return fuzzy_search(expected.as_str(), value.as_str()) < MAX_SIFT_DISTANCE,
            FilterOperator::Contains => return value.contains(expected.as_str()),
            FilterOperator::NotContains => return !value.contains(expected.as_str()),
            FilterOperator::Empty => return false,
            FilterOperator::NotEmpty => return true,
            _ => ()
        }
    }
    filter.operator == FilterOperator::Empty
}

pub fn filter_date(filter: &DateFilter, value: &Value) -> bool {
    let o = find_string(filter.field.as_str(), value);
    if let Some(value) = o {
        match DateTime::parse_from_rfc3339(value.as_str()) {
            Err(_) => {
                return false
            }
            Ok(time_in_json) => {
                let time_in_json = time_in_json.with_timezone(&UTC);
                match filter.operator {
                    FilterOperator::Equal => return filter.datetime.unwrap() == time_in_json,
                    FilterOperator::NotEqual => return filter.datetime.unwrap() != time_in_json,
                    FilterOperator::GreaterEquals => return time_in_json >= filter.datetime.unwrap(),
                    FilterOperator::GreaterThan => return time_in_json>filter.datetime.unwrap(),
                    FilterOperator::LessEquals => return time_in_json <= filter.datetime.unwrap(),
                    FilterOperator::LessThan => return time_in_json < filter.datetime.unwrap(),
                    FilterOperator::Empty => return false,
                    FilterOperator::NotEmpty => return true,
                    _ => unreachable!()
                }
            }
        }
    }
    filter.operator == FilterOperator::Empty
}

pub fn filter_date_time(filter: &DateFilter, time: &DateTime<UTC>) -> bool{
    match filter.operator {
        FilterOperator::Equal => return &filter.datetime.unwrap() == time,
        FilterOperator::NotEqual => return &filter.datetime.unwrap() != time,
        FilterOperator::GreaterEquals => return time >= &filter.datetime.unwrap(),
        FilterOperator::GreaterThan => return time>&filter.datetime.unwrap(),
        FilterOperator::LessEquals => return time <= &filter.datetime.unwrap(),
        FilterOperator::LessThan => return time < &filter.datetime.unwrap(),
        FilterOperator::Empty => return false,
        FilterOperator::NotEmpty => return true,
        _ => unreachable!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use super::super::searchparam::SearchFilter;

    fn get_task() -> Value {
        json!({
          "title": "My title",
          "name": "My name",
          "tags": [
            "bla",
            "blubb"
          ],
          "created": "2017-06-08T18:45:01.123+01:00",
          "details": {
            "context": "home",
            "priority": "asap",
          }
        })
    }

    #[test]
    fn test_find_string() {
        let task = get_task();
        println!("## search for priority");
        assert_eq!("asap", find_string("priority", &task).unwrap());
        println!("## search for details");
        assert_eq!(None, find_string("details", &task));
        println!("## search for name");
        assert_eq!("My name", find_string("name", &task).unwrap());
        println!("## search for tags");
        assert_eq!(None, find_string("tags", &task));
    }

    #[test]
    fn test_fuzzy_search() {
        assert_that(&fuzzy_search("steak", "I lvoe stekas!")).is_less_than_or_equal_to(3f32);
        assert_that(&fuzzy_search("steak", "I lvoe steaks!")).is_less_than_or_equal_to(1f32);
        assert_that(&fuzzy_search("steak", "I lvoe Steak !")).is_equal_to(0f32);
        assert_that(&fuzzy_search("steak", "I lvoe Steak!")).is_equal_to(0f32);
    }

    #[test]
    fn text_filtering() {
        let json = get_task();
        assert!(TextFilter::new(FilterOperator::FuzzyContains, Some("nmae".into()), "name").test(&json));
        assert!(TextFilter::new(FilterOperator::Contains, Some("nam".into()), "name").test(&json));
        assert!(TextFilter::new(FilterOperator::NotContains, Some("nme".into()), "name").test(&json));
        assert!(TextFilter::new(FilterOperator::Equal, Some("my name".into()), "name").test(&json));
        assert!(TextFilter::new(FilterOperator::NotEqual, Some("Your name".into()), "name").test(&json));
        assert!(TextFilter::new(FilterOperator::Empty, None, "NotFound").test(&json));
        assert!(TextFilter::new(FilterOperator::NotEmpty, Some("nmae".into()), "name").test(&json));
    }

    #[test]
    fn date_filtering() {
        let json = get_task();

        let exact_date = UTC.ymd(2017, 6, 8).and_hms_milli(17, 45, 1, 123);
        let parsed = DateTime::parse_from_rfc3339("2017-06-08T18:45:01.123+01:00").unwrap().with_timezone(&UTC);
        assert_eq!(exact_date, parsed);

        assert!(DateFilter::new(FilterOperator::Equal, Some(exact_date.clone()), "created").test(&json));
        assert!(DateFilter::new(FilterOperator::NotEqual, Some(exact_date - Duration::minutes(3)), "created").test(&json));
        assert!(DateFilter::new(FilterOperator::Empty, None, "NotFound").test(&json));
        assert!(DateFilter::new(FilterOperator::NotEmpty, None, "created").test(&json));
        assert!(DateFilter::new(FilterOperator::GreaterThan, Some(exact_date - Duration::minutes(3)), "created").test(&json));
        assert!(DateFilter::new(FilterOperator::GreaterEquals, Some(exact_date.clone()), "created").test(&json));
        assert!(DateFilter::new(FilterOperator::LessThan, Some(exact_date + Duration::minutes(3)), "created").test(&json));
        assert!(DateFilter::new(FilterOperator::LessEquals, Some(exact_date.clone()), "created").test(&json));
    }
}