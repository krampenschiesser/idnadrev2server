// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, UTC};
use serde_json::Value;
use std::fmt::Debug;
use super::filter::{filter_text, filter_date};

pub trait SearchFilter: Debug {
    fn test(&self, value: &Value) -> bool;

    fn filter_operator_valid(operator: &FilterOperator) -> bool;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchParam {
    pub offset: u32,
    pub limit: u32,

    pub any: Option<String>,

    pub name: Option<String>,
    pub file_type: Option<String>,
    pub tags: Vec<String>,

    pub created: Option<DateFilter>,
    pub updated: Option<DateFilter>,
    pub deleted: Option<DateFilter>,

    pub text_filters: Vec<TextFilter>,
    pub date_filters: Vec<DateFilter>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    FuzzyContains,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterEquals,
    LessEquals,
    Empty,
    NotEmpty,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DateFilter {
    pub datetime: Option<DateTime<UTC>>,
    pub field: String,
    pub operator: FilterOperator,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextFilter {
    pub text: Option<String>,
    pub field: String,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueryParamError {
    EmptyParameterFound,
    InvalidParameter(String),
    InvalidFilterOperator(String),
    InvalidDateTime(String),
    InvalidOperatorForText(String, FilterOperator),
    InvalidOperatorForDate(String, FilterOperator),
    InvalidDate { error: String, date: String },
    InvalidNumber(String),
}

struct QueryParam {
    key: String,
    value: String,
}

impl FilterOperator {
    fn no_value(&self) -> bool {
        match self {
            &FilterOperator::NotEmpty | &FilterOperator::Empty => true,
            _ => false
        }
    }
}


impl QueryParam {
    pub fn new(key: &str, value: &str) -> Self {
        QueryParam { key: key.to_string(), value: value.to_string() }
    }

    pub fn has_operator(&self) -> bool {
        let offset = 2;
        let offset = if self.is_date_search() { offset + "date:".len() } else { offset };
        let o = self.value.as_str().chars().nth(offset);
        if let Some(char) = o {
            if char == ':' {
                return true;
            }
        }
        return false;
    }

    pub fn get_operator(&self) -> Result<FilterOperator, QueryParamError> {
        let offset = 2;
        let offset = if self.is_date_search() { offset + "date:".len() } else { offset };
        let o = self.value.as_str().chars().nth(offset);
        if let Some(char) = o {
            if char == ':' {
                let offset = 0;
                let offset = if self.is_date_search() { offset + "date:".len() } else { offset };
                let substr: &str = &self.value.as_str()[offset..offset + 2];

                return match substr {
                    "eq" => Ok(FilterOperator::Equal),
                    "ne" => Ok(FilterOperator::NotEqual),
                    "fc" => Ok(FilterOperator::FuzzyContains),
                    "ct" => Ok(FilterOperator::Contains),
                    "nc" => Ok(FilterOperator::NotContains),
                    "nl" => Ok(FilterOperator::Empty),
                    "nn" => Ok(FilterOperator::NotEmpty),
                    "gt" => Ok(FilterOperator::GreaterThan),
                    "ge" => Ok(FilterOperator::GreaterEquals),
                    "lt" => Ok(FilterOperator::LessThan),
                    "le" => Ok(FilterOperator::LessEquals),
                    e => Err(QueryParamError::InvalidFilterOperator(e.into()))
                };
            }
        }
        if self.is_date_search() {
            Ok(FilterOperator::Equal)
        } else {
            Ok(FilterOperator::FuzzyContains)
        }
    }

    pub fn is_offset(&self) -> bool {
        self.key == "offset"
    }
    pub fn is_limit(&self) -> bool {
        self.key == "limit"
    }
    pub fn is_any(&self) -> bool {
        self.key == "any"
    }
    pub fn is_name(&self) -> bool {
        self.key == "name"
    }
    pub fn is_type(&self) -> bool {
        self.key == "type"
    }
    pub fn is_tags(&self) -> bool {
        self.key == "tags"
    }
    pub fn is_created(&self) -> bool {
        self.key == "created"
    }
    pub fn is_updated(&self) -> bool {
        self.key == "updated"
    }
    pub fn is_deleted(&self) -> bool {
        self.key == "deleted"
    }

    pub fn is_date_search(&self) -> bool {
        let check = "date:";
        if self.value.len() > check.len() {
            &self.value.as_str()[0..check.len()] == check
        } else {
            false
        }
    }

    pub fn get_text_value(&self) -> String {
        if self.is_date_search() {
            let offset = "date:".len();
            if self.has_operator() {
                self.value.as_str()[offset + 3..].to_string()
            } else {
                self.value.as_str()[offset..].to_string()
            }
        } else {
            if self.has_operator() {
                self.value.as_str()[3..].to_string()
            } else {
                self.value.clone()
            }
        }
    }
    pub fn get_date_value(&self) -> Result<DateTime<UTC>, QueryParamError> {
        if self.is_date_search() {
            let date_text = self.get_text_value();

            match DateTime::parse_from_rfc3339(date_text.as_str()) {
                Err(e) => Err(QueryParamError::InvalidDate { error: format!("{}", e), date: date_text }),
                Ok(fixed_offset) => Ok(fixed_offset.with_timezone(&UTC))
            }
        } else {
            Err(QueryParamError::InvalidDateTime(self.value.clone()))
        }
    }

    pub fn get_u32_value(&self) -> Result<u32, QueryParamError> {
        match self.value.parse::<u32>() {
            Err(e) => Err(QueryParamError::InvalidNumber(self.value.clone())),
            Ok(number) => Ok(number)
        }
    }
}

impl SearchFilter for TextFilter {
    fn test(&self, value: &Value) -> bool {
        filter_text(&self, &value)
    }

    fn filter_operator_valid(operator: &FilterOperator) -> bool {
        use self::FilterOperator::*;

        match operator {
            &Equal | &NotEqual | &FuzzyContains | &Contains | &NotContains | &Empty | &NotEmpty => true,
            _ => false
        }
    }
}

impl TextFilter {
    pub fn new(operator: FilterOperator, text: Option<String>, field: &str) -> Self {
        TextFilter { text: text, field: field.to_string(), operator: operator }
    }


    fn from_param(param: QueryParam) -> Result<Self, QueryParamError> {
        if param.is_date_search() {
            return Err(QueryParamError::InvalidParameter(param.key));
        }

        let operator = param.get_operator()?;
        if Self::filter_operator_valid(&operator) {
            let f = if operator.no_value() {
                TextFilter { operator: operator, field: param.key, text: None }
            } else {
                let val = param.get_text_value();
                TextFilter { operator: operator, field: param.key, text: Some(val) }
            };
            Ok(f)
        } else {
            Err(QueryParamError::InvalidOperatorForText(param.value, operator))
        }
    }
}

impl SearchFilter for DateFilter {
    fn test(&self, value: &Value) -> bool {
        filter_date(&self, &value)
    }

    fn filter_operator_valid(operator: &FilterOperator) -> bool {
        use self::FilterOperator::*;
        match operator {
            &Contains | &NotContains | &FuzzyContains => false,
            _ => true
        }
    }
}

impl DateFilter {
    pub fn new(operator: FilterOperator, datetime: Option<DateTime<UTC>>, field: &str) -> Self {
        DateFilter { datetime: datetime, field: field.to_string(), operator: operator }
    }

    fn from_param(param: QueryParam) -> Result<Self, QueryParamError> {
        if !param.is_date_search() {
            return Err(QueryParamError::InvalidParameter(param.key));
        }

        let operator = param.get_operator()?;
        if Self::filter_operator_valid(&operator) {
            let f = if operator.no_value() {
                DateFilter { operator: operator, datetime: None, field: param.key }
            } else {
                let val = param.get_date_value()?;
                DateFilter { operator: operator, datetime: Some(val), field: param.key }
            };
            Ok(f)
        } else {
            Err(QueryParamError::InvalidOperatorForText(param.value, operator))
        }
    }
}

impl SearchParam {
    pub fn new() -> Self {
        SearchParam { offset: 0, limit: 25, any: None, name: None, file_type: None, tags: Vec::new(), created: None, updated: None, deleted: None, text_filters: Vec::new(), date_filters: Vec::new() }
    }

    fn parse_str(param: &str) -> Result<Vec<QueryParam>, QueryParamError> {
        let mut retval = Vec::new();
        let offset = if param.starts_with("?") { 1 } else { 0 };
        let split = param[offset..].split('&');
        for param_string in split {
            let pair: Vec<&str> = param_string.split('=').collect();
            if pair.len() != 2 {
                return Err(QueryParamError::InvalidParameter(param_string.to_string()));
            }
            retval.push(QueryParam { key: pair[0].to_string(), value: pair[1].to_string() })
        }
        Ok(retval)
    }

    pub fn from_query_param(query_param: &str) -> Result<Self, QueryParamError> {
        let mut retval = Self::new();
        if query_param.len() <= 1 {
            return Ok(retval);
        }

        let params = Self::parse_str(query_param)?;
        for param in params {
            if param.is_offset() {
                retval.offset = param.get_u32_value()?;
            } else if param.is_limit() {
                retval.limit = param.get_u32_value()?;
            } else if param.is_any() {
                retval.any = Some(param.value);
            } else if param.is_name() {
                retval.name = Some(param.value);
            } else if param.is_type() {
                retval.file_type = Some(param.value);
            } else if param.is_tags() {
                let tags: Vec<String> = param.value.split(',').map(|s| s.to_string()).collect();
                retval.tags = tags;
            } else if param.is_created() {
                retval.created = Some(DateFilter::from_param(param)?);
            } else if param.is_updated() {
                retval.updated = Some(DateFilter::from_param(param)?);
            } else if param.is_deleted() {
                retval.deleted = Some(DateFilter::from_param(param)?);
            } else if param.is_date_search() {
                retval.date_filters.push(DateFilter::from_param(param)?);
            } else {
                retval.text_filters.push(TextFilter::from_param(param)?);
            }
        }

        Ok(retval)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_param_date() {
        assert!(QueryParam::new("key", "date:value").is_date_search());
    }

    #[test]
    fn query_param_operators() {
        assert_eq!(FilterOperator::NotEqual, QueryParam::new("key", "date:ne:value").get_operator().unwrap());

        assert_eq!(FilterOperator::FuzzyContains, QueryParam::new("key", "date").get_operator().unwrap());
        assert_eq!(FilterOperator::Equal, QueryParam::new("key", "eq:value").get_operator().unwrap());
        assert_eq!(FilterOperator::NotEqual, QueryParam::new("key", "ne:value").get_operator().unwrap());
        assert_eq!(FilterOperator::Empty, QueryParam::new("key", "nl:value").get_operator().unwrap());
        assert_eq!(FilterOperator::NotEmpty, QueryParam::new("key", "nn:value").get_operator().unwrap());
        assert_eq!(FilterOperator::Contains, QueryParam::new("key", "ct:value").get_operator().unwrap());
        assert_eq!(FilterOperator::NotContains, QueryParam::new("key", "nc:value").get_operator().unwrap());
        assert_eq!(FilterOperator::GreaterThan, QueryParam::new("key", "gt:value").get_operator().unwrap());
        assert_eq!(FilterOperator::GreaterEquals, QueryParam::new("key", "ge:value").get_operator().unwrap());
        assert_eq!(FilterOperator::LessThan, QueryParam::new("key", "lt:value").get_operator().unwrap());
        assert_eq!(FilterOperator::LessEquals, QueryParam::new("key", "le:value").get_operator().unwrap());

        let res = QueryParam::new("key", "bl:value").get_operator();
        match res {
            Err(QueryParamError::InvalidFilterOperator(val)) => assert_eq!("bl", val),
            _ => panic!("Should not be valid filter operator")
        }
    }

    #[test]
    fn invalid_length() {
        SearchParam::from_query_param("?").unwrap();
        SearchParam::from_query_param("").unwrap();
    }

    #[test]
    fn query_param_text() {
        let param = SearchParam::from_query_param("?bla=blubb&huhu=haha").unwrap();
        assert_eq!(2, param.text_filters.len());
    }

    use chrono::prelude::*;

    #[test]
    fn search_param_date() {
        let param = SearchParam::from_query_param("?bla=date:gt:2016-04-03T16:33:27+03:00").unwrap();
        assert_eq!(1, param.date_filters.len());
        let ref date_filter = param.date_filters[0];
        let utc_date = UTC.ymd(2016, 4, 3).and_hms(13, 33, 27);
        assert_eq!(utc_date, date_filter.datetime.unwrap());
    }

    #[test]
    fn invalid_param() {
        let res = SearchParam::from_query_param("?bla=blubb&huhu");
        match res {
            Err(QueryParamError::InvalidParameter(val)) => assert_eq!("huhu".to_string(), val),
            _ => panic!("Should have failed to parse")
        }
    }

    #[test]
    fn any() {
        let search = SearchParam::from_query_param("?bla=blubb&any=huhu").unwrap();
        assert_eq!("huhu", search.any.unwrap());
    }

    #[test]
    fn limit() {
        let search = SearchParam::from_query_param("?bla=blubb&limit=42").unwrap();
        assert_eq!(42u32, search.limit);
    }

    #[test]
    fn offset() {
        let search = SearchParam::from_query_param("?offset=42").unwrap();
        assert_eq!(42u32, search.offset);
    }

    #[test]
    fn tags() {
        let search = SearchParam::from_query_param("?tags=bla,blubb,huhu").unwrap();
        assert_eq!(vec!["bla".to_string(), "blubb".to_string(), "huhu".to_string()], search.tags);
    }

    #[test]
    fn title() {
        let search = SearchParam::from_query_param("?name=title").unwrap();
        assert_eq!("title", search.name.unwrap());
    }

    #[test]
    fn deleted() {
        let search = SearchParam::from_query_param("?deleted=date:2017-05-01T12:03:03+01:00").unwrap();
        assert_eq!(UTC.ymd(2017, 5, 1).and_hms(11, 3, 3), search.deleted.unwrap().datetime.unwrap());
    }

    #[test]
    fn empty() {
        let search = SearchParam::from_query_param("?deleted=date:nl:").unwrap();
        assert_eq!(FilterOperator::Empty, search.deleted.unwrap().operator);
    }
}