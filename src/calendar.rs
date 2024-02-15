use chrono::{Datelike, Days, Duration, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

///
///
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
    /// The week of the year this entry applies to
    pub week_of_year: Option<u32>,
    /// The weekdays this entry applies to
    pub days_of_week: Option<Vec<Weekday>>,
    /// The offset can be used to model lieu days, where a holiday would
    /// normally fall on a weekend and a replacement should be offered.
    pub offset: i32,
}

impl<T> Calendar<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn classify(self, date: NaiveDate) -> Option<T> {
        for e in self.entries {
            if e.matches(date) {
                return Some(e.classification);
            }
        }
        None
    }

    pub fn classify_ymd(self, year: i32, month: u32, day: u32) -> Option<T> {
        let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        self.classify(d)
    }

    pub fn add_entry(
        &mut self,
        classification: T,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        week_of_year: Option<u32>,
        week_of_month: Option<i32>,
        days_of_week: Option<Vec<Weekday>>,
        offset: i32,
    ) {
        let entry = CalendarEntry {
            year,
            month,
            day,
            week_of_month,
            week_of_year,
            days_of_week,
            classification,
            offset,
        };
        self.entries.insert(0, entry);
    }

    pub fn add_entry_ymd(
        &mut self,
        classification: T,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
    ) {
        self.add_entry(classification, year, month, day, None, None, None, 0)
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
            && self.week_of_month.map_or(true, |w| {
                if w > 0 {
                    w == ((d0 / 7) + 1) as i32
                } else if w < 0 {
                    let som = NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
                        .unwrap()
                        .checked_add_months(Months::new(1))
                        .unwrap();
                    let diff = (som - date).num_days();
                    -w == (((diff - 1) / 7) + 1) as i32
                } else {
                    false
                }
            })
            && self
                .days_of_week
                .clone()
                .map_or(true, |v| v.contains(&weekday))
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
        subject.add_entry(Day::Workday, None, None, None, None, None, None, 0);
        subject.add_entry(
            Day::Weekend,
            None,
            None,
            None,
            None,
            None,
            Some(vec![Weekday::Sun, Weekday::Sat]),
            0,
        );
        subject.add_entry(
            Day::BankHoliday,
            None,
            Some(5),
            None,
            None,
            Some(1),
            Some(vec![Weekday::Mon]),
            0,
        );
        subject.add_entry(
            Day::BankHoliday,
            None,
            Some(5),
            None,
            None,
            Some(-1),
            Some(vec![Weekday::Mon]),
            0,
        );

        let actual = subject.classify_ymd(y, m, d);

        assert_eq!(actual, expected);
    }

    #[test_case(2021, 12, 25, Day::Holiday("X-mas"))]
    #[test_case(2021, 12, 26, Day::Holiday("X-ing"))]
    #[test_case(2021, 12, 27, Day::Holiday("X-mas lieu"))]
    #[test_case(2021, 12, 28, Day::Holiday("X-ing lieu"))]
    fn test_offset_date(year: i32, month: u32, day: u32, expected: Day) {
        let mut subject = Calendar::<Day>::new();
        subject.add_entry(Day::Workday, None, None, None, None, None, None, 0);
        subject.add_entry(
            Day::Weekend,
            None,
            None,
            None,
            None,
            None,
            Some(vec![Weekday::Sun, Weekday::Sun]),
            0,
        );
        subject.add_entry_ymd(Day::Holiday("X-mas"), None, Some(12), Some(25));
        subject.add_entry_ymd(Day::Holiday("X-ing"), None, Some(12), Some(26));
        subject.add_entry(
            Day::Holiday("X-mas lieu"),
            None,
            Some(12),
            Some(25),
            None,
            None,
            Some(vec![Weekday::Sat, Weekday::Sun]),
            2,
        );
        subject.add_entry(
            Day::Holiday("X-ing lieu"),
            None,
            Some(12),
            Some(26),
            None,
            None,
            Some(vec![Weekday::Sat, Weekday::Sun]),
            2,
        );

        let actual = subject.classify_ymd(year, month, day);

        assert_eq!(actual, Some(expected));
    }
}
