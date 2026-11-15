use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageKind {
    Fixtures,
    Results,
    Table,
    Team,
    Fixture,
}

impl Display for PageKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Fixtures => "fixtures",
            Self::Results => "results",
            Self::Table => "table",
            Self::Team => "team",
            Self::Fixture => "fixture",
        };
        formatter.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueField {
    Row,
    KickOff,
    Outcome,
    Competition,
    Standing,
    Coordinates,
}

impl Display for IssueField {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Row => "row",
            Self::KickOff => "kick-off",
            Self::Outcome => "outcome",
            Self::Competition => "competition",
            Self::Standing => "standing",
            Self::Coordinates => "coordinates",
        };
        formatter.write_str(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIssue {
    pub page: PageKind,
    pub subject: String,
    pub field: IssueField,
    pub raw: String,
}

impl ParseIssue {
    pub(crate) fn new(page: PageKind, subject: &str, field: IssueField, raw: &str) -> Self {
        Self {
            page,
            subject: subject.to_owned(),
            field,
            raw: raw.to_owned(),
        }
    }
}

impl Display for ParseIssue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unrecognised {} for {} on the {} page: {:?}",
            self.field, self.subject, self.page, self.raw
        )
    }
}

#[derive(Debug)]
pub struct Parsed<T> {
    pub content: T,
    pub issues: Vec<ParseIssue>,
}

#[derive(Debug)]
enum ErrorKind {
    MissingElement(&'static str),
    InvalidSelector { css: &'static str, reason: String },
}

#[derive(Debug)]
pub struct PageError {
    page: PageKind,
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl PageError {
    pub(crate) fn missing_element(page: PageKind, element: &'static str) -> Self {
        Self {
            page,
            kind: ErrorKind::MissingElement(element),
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn invalid_selector(page: PageKind, css: &'static str, reason: String) -> Self {
        Self {
            page,
            kind: ErrorKind::InvalidSelector { css, reason },
            backtrace: Backtrace::capture(),
        }
    }

    pub fn page(&self) -> PageKind {
        self.page
    }

    pub fn is_missing_element(&self) -> bool {
        matches!(self.kind, ErrorKind::MissingElement(_))
    }
}

impl Display for PageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::MissingElement(element) => {
                write!(formatter, "the {} page has no {element}", self.page)?;
            }
            ErrorKind::InvalidSelector { css, reason } => {
                write!(
                    formatter,
                    "invalid selector {css} for the {} page: {reason}",
                    self.page
                )?;
            }
        }
        write!(formatter, "\n{}", self.backtrace)
    }
}

impl std::error::Error for PageError {}
