use chrono::{Datelike, Local, NaiveDate};
use leptos::prelude::*;

#[component]
pub fn CalendarPage() -> impl IntoView {
    let today = Local::now().date_naive();
    let displayed_year = RwSignal::new(today.year());
    let displayed_month = RwSignal::new(today.month());

    let go_to_previous_month = move |_: web_sys::MouseEvent| {
        let month = displayed_month.get_untracked();
        if month == 1 {
            displayed_year.update(|year| *year -= 1);
            displayed_month.set(12);
        } else {
            displayed_month.update(|month| *month -= 1);
        }
    };

    let go_to_next_month = move |_: web_sys::MouseEvent| {
        let month = displayed_month.get_untracked();
        if month == 12 {
            displayed_year.update(|year| *year += 1);
            displayed_month.set(1);
        } else {
            displayed_month.update(|month| *month += 1);
        }
    };

    let month_title = Memo::new(move |_| {
        NaiveDate::from_ymd_opt(displayed_year.get(), displayed_month.get(), 1)
            .unwrap()
            .format("%B %Y")
            .to_string()
    });

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="bg-white rounded-xl shadow-md p-8 w-full max-w-md space-y-4">
                <div class="flex items-center justify-between">
                    <button
                        class="p-2 rounded-lg hover:bg-gray-100 text-gray-600 text-xl leading-none"
                        on:click=go_to_previous_month
                    >
                        "‹"
                    </button>
                    <h2 class="text-lg font-semibold text-gray-800">
                        {month_title}
                    </h2>
                    <button
                        class="p-2 rounded-lg hover:bg-gray-100 text-gray-600 text-xl leading-none"
                        on:click=go_to_next_month
                    >
                        "›"
                    </button>
                </div>

                <div class="grid grid-cols-7">
                    {["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
                        .iter()
                        .map(|day_name| view! {
                            <div class="text-center text-xs font-medium text-gray-400 py-2">
                                {*day_name}
                            </div>
                        })
                        .collect_view()
                    }
                </div>

                <div class="grid grid-cols-7">
                    {move || {
                        let year = displayed_year.get();
                        let month = displayed_month.get();
                        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                        let weekday_offset = first_day.weekday().num_days_from_monday() as usize;
                        let days_in_month = (if month == 12 {
                            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                        } else {
                            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
                        })
                        .signed_duration_since(first_day)
                        .num_days() as u32;

                        let mut cells: Vec<Option<u32>> = vec![None; weekday_offset];
                        cells.extend((1..=days_in_month).map(Some));
                        while cells.len() % 7 != 0 {
                            cells.push(None);
                        }

                        cells
                            .into_iter()
                            .map(|day_opt| {
                                let is_today = day_opt.map_or(false, |day| {
                                    year == today.year()
                                        && month == today.month()
                                        && day == today.day()
                                });
                                let cell_class = if is_today {
                                    "flex items-center justify-center text-sm h-9 w-9 mx-auto rounded-full bg-blue-600 text-white font-semibold"
                                } else if day_opt.is_some() {
                                    "flex items-center justify-center text-sm h-9 w-9 mx-auto rounded-full text-gray-700 hover:bg-gray-100"
                                } else {
                                    "h-9"
                                };
                                view! {
                                    <div class=cell_class>
                                        {day_opt.map(|day| day.to_string())}
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        </main>
    }
}
