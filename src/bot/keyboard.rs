use chrono::Local;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn render_attendance(
    students: &[(i64, String, bool)],
    is_admin: bool,
    current_user_tg_id: i64,
) -> (String, InlineKeyboardMarkup) {
    let today = Local::now().format("%d.%m.%Y").to_string();
    let mut text = format!("Посещаемость на {}\n\n", today);
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for (tg_id, name, present) in students {
        let status_emoji = if *present { "🟢" } else { "🔴" };
        let status_text = if *present { "присутствовал" } else { "отсутствовал" };
        text.push_str(&format!("{} {} — {}\n", status_emoji, name, status_text));

       
        if !is_admin && *tg_id == current_user_tg_id {
            let button_text = if *present {
                "Отметить отсутствие"
            } else {
                "Отметить присутствие"
            };
            buttons.push(vec![InlineKeyboardButton::callback(
                button_text,
                format!("toggle_{}", tg_id),
            )]);
        }
    }

    if students.is_empty() {
        text.push_str("Список пуст.\nЗарегистрируйтесь через /start.");
    }

    if is_admin {
      
        if let Some((_, _, present)) = students.iter().find(|&&(tg, _, _)| tg == current_user_tg_id) {
            let button_text = if *present {
                "Отметить отсутствие"
            } else {
                "Отметить присутствие"
            };
            buttons.push(vec![InlineKeyboardButton::callback(button_text, "toggle_self")]);
        }

        // Кнопки "Отметить всех" и "Обновить"
        buttons.push(vec![
            InlineKeyboardButton::callback(" Отметить всех", "show_student_selector"),
            InlineKeyboardButton::callback(" Обновить", "refresh"),
        ]);
    }

    (text, InlineKeyboardMarkup::new(buttons))
}

pub fn render_student_selector(students: &[(i64, String, bool)]) -> (String, InlineKeyboardMarkup) {
    let mut text = "Выберите студента для отметки:\n\n".to_string();
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for (tg_id, name, present) in students {
        let status = if *present { "🟢" } else { "🔴" };
        text.push_str(&format!("{} {}\n", status, name));

        buttons.push(vec![InlineKeyboardButton::callback(
            name.clone(),
            format!("admin_toggle_{}", tg_id),
        )]);
    }

    buttons.push(vec![InlineKeyboardButton::callback("⬅️ Назад", "back_to_main")]);

    (text, InlineKeyboardMarkup::new(buttons))
}