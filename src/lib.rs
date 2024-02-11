use chrono::{Datelike, NaiveDate, Weekday};

#[derive(Debug)]
pub struct Calendar<T> {
    pub entries: Vec<CalendarEntry<T>>,
}

impl<T> Calendar<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn classify(self, date: NaiveDate) -> Option<T> {
        let y = date.year();
        let m = date.month0() + 1;
        let d = date.day0() + 1;
        let w = d / 7 + 1;
        let wd = date.weekday();
        for e in self.entries {
            if e.matches(y, m, d, w, &wd) {
                return Some(e.classification);
            }
        }
        None
    }

    pub fn add_entry(
        &mut self,
        classification: T,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        week: Option<u32>,
        days_of_week: Option<Vec<Weekday>>,
    ) {
        let entry = CalendarEntry {
            year,
            month,
            day,
            week,
            days_of_week,
            classification,
        };
        self.entries.insert(0, entry);
    }
}

#[derive(Debug)]
pub struct CalendarEntry<T> {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub week: Option<u32>,
    pub days_of_week: Option<Vec<Weekday>>,
    pub classification: T,
}

impl<T> CalendarEntry<T> {
    fn matches(&self, year: i32, month: u32, day: u32, week: u32, weekday: &Weekday) -> bool {
        self.year.unwrap_or(year) == year
            && self.month.unwrap_or(month) == month
            && self.day.unwrap_or(day) == day
            && self.week.unwrap_or(week) == week
            && self
                .days_of_week
                .clone()
                .map_or(true, |v| v.contains(weekday))
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
    }

    #[test_case(2024, 2, 11; "Today")]
    #[test_case(2000, 1, 11; "Today in 2000")]
    #[test_case(1955, 11, 5; "Back to the Future")]
    #[test_case(2015, 11, 5; "Back to the Future II")]
    fn empty_calendar_returns_nothing(y: i32, m: u32, d: u32) {
        let subject = Calendar::<Day>::new();
        let expected: Option<Day> = None;

        let day = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let actual = subject.classify(day);

        assert_eq!(actual, expected);
    }

    #[test_case(2024, 2, 5, Some(Day::Workday); "Weekday")]
    #[test_case(2024, 2, 11, Some(Day::Weekend); "Weekend")]
    #[test_case(2024, 5, 6, Some(Day::BankHoliday); "Bank Holiday")]
    fn test_a_calender_with_entries(y: i32, m: u32, d: u32, expected: Option<Day>) {
        let mut subject = Calendar::<Day>::new();
        subject.add_entry(Day::Workday, None, None, None, None, None);
        subject.add_entry(
            Day::Weekend,
            None,
            None,
            None,
            None,
            Some(vec![Weekday::Sun, Weekday::Sat]),
        );
        subject.add_entry(
            Day::BankHoliday,
            None,
            Some(5),
            None,
            Some(1),
            Some(vec![Weekday::Mon]),
        );

        let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let actual = subject.classify(date);

        assert_eq!(actual, expected);
    }
}
