use std::cmp::Ordering;

use chrono::{Datelike, Duration, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

///
/// The LSTC Calendar is used to create a collection of patterns to perform
/// matches against, thereby allowing us to classify dates into more useful
/// domain meanings, such as "workday", "weekend" or "bank holiday", etc.
/// ```
/// use chrono::{NaiveDate, Weekday};
/// use lstc_calendar::{Calendar, CalendarEntry};
/// enum Day {
///     Workday,
///     Weekend,
///     BankHoliday(&'static str),
/// }
/// let mut subject = Calendar::<Day>::new();
/// subject.add(CalendarEntry::all(Day::Workday));
/// subject.add(CalendarEntry::days(Day::Weekend, vec![Weekday::Sat, Weekday::Sun]));
/// subject.add(CalendarEntry::ymd(Day::BankHoliday("Christmas"), None, Some(12), Some(25)));
/// let today = chrono::offset::Local::now().naive_local().date();
/// let today = subject.classify(today);
/// ```
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Calendar<T> {
    pub entries: Vec<CalendarEntry<T>>,
}

///
///
///
#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEntry<T> {
    /// How to classify this entry
    pub classification: T,
    /// The year that this entry applies to
    pub year: Option<i32>,
    /// The month that this entry applies to
    pub month: Option<u32>,
    /// The day of the month this entry applies to
    pub day: Option<u32>,
    /// The week of the month this entry applies to
    pub week_of_month: Option<i32>,
    /// The weekdays this entry applies to
    pub days_of_week: Option<Vec<Weekday>>,
    /// The offset can be used to model lieu days, where a holiday would
    /// normally fall on a weekend and a replacement should be offered.
    pub offset: i32,
}

impl<T> std::default::Default for Calendar<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Calendar<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    ///
    /// Classifify a given date, based on the entries added.  Entries are
    /// evaluated in reverse order, making the latest entries evaludated first.
    ///
    pub fn classify(self, date: NaiveDate) -> Option<T> {
        for e in self.entries {
            if e.matches(date) {
                return Some(e.classification);
            }
        }
        None
    }

    ///
    /// Helper function that classifies a date from it's ymd portions.
    ///
    pub fn classify_ymd(self, year: i32, month: u32, day: u32) -> Option<T> {
        let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        self.classify(d)
    }

    ///
    /// Add a Calendar Entry into the Calendar.  Ordering is important when
    /// classifying dates.  The oldest entry is matched last, therefore allowing
    /// greater specificity in further entries.
    ///
    pub fn add(&mut self, entry: CalendarEntry<T>) {
        self.entries.insert(0, entry)
    }
}

impl<T> CalendarEntry<T> {
    fn matches(&self, date: NaiveDate) -> bool {
        let date = date - Duration::days(self.offset as i64);
        let year = date.year();
        let month = date.month0() + 1;
        let d0 = date.day0();
        let day = d0 + 1;
        let weekday = date.weekday();

        self.year.map_or(true, |y| y == year)
            && self.month.map_or(true, |m| m == month)
            && self.day.map_or(true, |d| d == day)
            && self.week_of_month.map_or(true, |w| match w.cmp(&0) {
                Ordering::Equal => false,
                Ordering::Greater => w == ((d0 / 7) + 1) as i32,
                Ordering::Less => {
                    let som = NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
                        .unwrap()
                        .checked_add_months(Months::new(1))
                        .unwrap();
                    let diff = (som - date).num_days();
                    -w == (((diff - 1) / 7) + 1) as i32
                }
            })
            && self
                .days_of_week
                .clone()
                .map_or(true, |v| v.contains(&weekday))
    }

    pub fn new(
        classification: T,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        week_of_month: Option<i32>,
        days_of_week: Option<Vec<Weekday>>,
        offset: i32,
    ) -> Self {
        Self {
            classification,
            year,
            month,
            day,
            week_of_month,
            days_of_week,
            offset,
        }
    }

    pub fn all(classification: T) -> Self {
        Self::new(classification, None, None, None, None, None, 0)
    }

    pub fn ymd(classification: T, year: Option<i32>, month: Option<u32>, day: Option<u32>) -> Self {
        Self::new(classification, year, month, day, None, None, 0)
    }

    pub fn ymd_offset(
        classification: T,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        offset: i32,
    ) -> Self {
        Self::new(classification, year, month, day, None, None, offset)
    }

    pub fn days(classification: T, days_of_week: Vec<Weekday>) -> Self {
        Self::new(
            classification,
            None,
            None,
            None,
            None,
            Some(days_of_week),
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[derive(PartialEq, Clone, Copy, Debug)]
    enum Day {
        Workday,
        Weekend,
        BankHoliday,
        Holiday(&'static str),
    }

    #[test_case(2024, 2, 11; "Today")]
    #[test_case(2000, 1, 11; "Today in 2000")]
    #[test_case(1955, 11, 5; "Back to the Future")]
    #[test_case(2015, 11, 5; "Back to the Future II")]
    #[test_case(1885, 11, 5; "Back to the Future III")]
    fn empty_calendar_returns_nothing(y: i32, m: u32, d: u32) {
        let subject = Calendar::<Day>::new();
        let expected: Option<Day> = None;

        let day = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let actual = subject.classify(day);

        assert_eq!(actual, expected);
    }

    #[test_case(2024, 2, 5, Some(Day::Workday); "Weekday")]
    #[test_case(2024, 2, 11, Some(Day::Weekend); "Weekend")]
    #[test_case(2024, 5, 6, Some(Day::BankHoliday); "Early May Bank Holiday")]
    #[test_case(2024, 5, 27, Some(Day::BankHoliday); "Late May Bank Holiday")]
    fn test_a_calender_with_entries(y: i32, m: u32, d: u32, expected: Option<Day>) {
        let mut subject = Calendar::<Day>::new();
        subject.add(CalendarEntry::all(Day::Workday));
        subject.add(CalendarEntry::days(
            Day::Weekend,
            vec![Weekday::Sun, Weekday::Sat],
        ));
        subject.add(CalendarEntry::new(
            Day::BankHoliday,
            None,
            Some(5),
            None,
            Some(1),
            Some(vec![Weekday::Mon]),
            0,
        ));
        subject.add(CalendarEntry::new(
            Day::BankHoliday,
            None,
            Some(5),
            None,
            Some(-1),
            Some(vec![Weekday::Mon]),
            0,
        ));

        let actual = subject.classify_ymd(y, m, d);

        assert_eq!(actual, expected);
    }

    #[test_case(2021, 12, 25, Day::Holiday("X-mas"))]
    #[test_case(2021, 12, 26, Day::Holiday("X-ing"))]
    #[test_case(2021, 12, 27, Day::Holiday("X-mas lieu"))]
    #[test_case(2021, 12, 28, Day::Holiday("X-ing lieu"))]
    fn test_offset_date(year: i32, month: u32, day: u32, expected: Day) {
        let mut subject = Calendar::<Day>::new();
        subject.add(CalendarEntry::all(Day::Workday));
        subject.add(CalendarEntry::days(
            Day::Weekend,
            vec![Weekday::Sun, Weekday::Sun],
        ));
        subject.add(CalendarEntry::ymd(
            Day::Holiday("X-mas"),
            None,
            Some(12),
            Some(25),
        ));
        subject.add(CalendarEntry::ymd(
            Day::Holiday("X-ing"),
            None,
            Some(12),
            Some(26),
        ));
        subject.add(CalendarEntry::new(
            Day::Holiday("X-mas lieu"),
            None,
            Some(12),
            Some(25),
            None,
            Some(vec![Weekday::Sat, Weekday::Sun]),
            2,
        ));
        subject.add(CalendarEntry::new(
            Day::Holiday("X-ing lieu"),
            None,
            Some(12),
            Some(26),
            None,
            Some(vec![Weekday::Sat, Weekday::Sun]),
            2,
        ));

        let actual = subject.classify_ymd(year, month, day);

        assert_eq!(actual, Some(expected));
    }
}
