mod assemble;
mod error;
mod fixture_list;
mod outcome;
mod table;
mod team;
mod text;
mod venue;

#[doc(inline)]
pub use assemble::{assemble, Pages};
#[doc(inline)]
pub use error::{IssueField, PageError, PageKind, ParseIssue, Parsed};
#[doc(inline)]
pub use fixture_list::{
    parse_fixtures_page, parse_results_page, CompetitionGroup, FixtureListPage, FixtureRow,
};
#[doc(inline)]
pub use table::{parse_table_page, TablePage, TableRow};
#[doc(inline)]
pub use team::{parse_team_page, TeamPage};
#[doc(inline)]
pub use text::normalised_name;
#[doc(inline)]
pub use venue::{parse_fixture_page, VenuePage};
