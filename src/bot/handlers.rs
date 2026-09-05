use teloxide::prelude::*;
use teloxide::types::ChatId;
use tracing::error;

use crate::bot::keyboard::{render_attendance, render_student_selector};
use crate::config::Config;
use crate::db::operations as repo;
use crate::db::DbConnection;
use crate::utils::is_admin;

use super::commands::Command;

async fn check_registration(db: &DbConnection, user_id: i64) -> Result<bool, anyhow::Error> {
    repo::is_student_registered(db, user_id).await
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: DbConnection,
    config: Config,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let user_id = msg.from().as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    let username = msg.from().as_ref().and_then(|u| u.username.as_deref());

    let is_reg = match check_registration(&db, user_id).await {
        Ok(v) => v,
        Err(e) => {
            error!("Ошибка проверки регистрации: {}", e);
            bot.send_message(chat_id, "⚠️ Не удалось проверить регистрацию.").await?;
            return Ok(());
        }
    };

    match cmd {
        Command::Start => {
            if !is_reg {
                bot.send_message(chat_id, "👋 Добро пожаловать!\n\nВведите ваше ФИО.").await?;
            } else {
                let is_admin = is_admin(username, &config);
                show_attendance(&bot, chat_id, &db, is_admin, user_id).await?;
            }
        }
        Command::Help => {
            let text = if is_admin(username, &config) {
                "Доступные команды:\n\n\
                 /start — начать работу\n\
                 /list — посмотреть посещаемость\n\
                 /stats — статистика\n\
                 /remove <tg_id> — удалить студента\n\
                 /reset_today — сбросить отметки за сегодня\n\
                 /help — помощь"
            } else {
                "Доступные команды:\n\n\
                 /start — начать работу\n\
                 /list — посмотреть посещаемость\n\
                 /help — помощь"
            };
            bot.send_message(chat_id, text).await?;
        }
        Command::List => {
            if !is_reg {
                bot.send_message(chat_id, "⚠️ Сначала зарегистрируйтесь через /start.").await?;
                return Ok(());
            }
            let is_admin = is_admin(username, &config);
            show_attendance(&bot, chat_id, &db, is_admin, user_id).await?;
        }
        Command::Stats => {
            if !is_admin(username, &config) {
                bot.send_message(chat_id, "❌ У вас нет прав администратора.").await?;
                return Ok(());
            }
            match repo::get_stats(&db).await {
                Ok((total, present)) => {
                    let absent = total - present;
                    let text = format!(
                        "📊 Статистика на сегодня:\n\nВсего: {}\nПрисутствуют: {}\nОтсутствуют: {}",
                        total, present, absent
                    );
                    bot.send_message(chat_id, text).await?;
                }
                Err(e) => {
                    error!("Ошибка получения статистики: {}", e);
                    bot.send_message(chat_id, "⚠️ Не удалось получить статистику.").await?;
                }
            }
        }
        Command::Remove { tg_id } => {
            if !is_admin(username, &config) {
                bot.send_message(chat_id, "❌ У вас нет прав администратора.").await?;
                return Ok(());
            }
            match repo::remove_student(&db, tg_id).await {
                Ok(_) => {
                    bot.send_message(chat_id, format!("✅ Студент с ID {} удалён.", tg_id)).await?;
                }
                Err(e) => {
                    error!("Ошибка удаления студента: {}", e);
                    bot.send_message(chat_id, format!("❌ Не удалось удалить студента: {}", e)).await?;
                }
            }
        }
        Command::ResetToday => {
            if !is_admin(username, &config) {
                bot.send_message(chat_id, "❌ У вас нет прав администратора.").await?;
                return Ok(());
            }
            match repo::reset_today_attendance(&db).await {
                Ok(_) => {
                    bot.send_message(chat_id, "✅ Все отметки за сегодня сброшены.").await?;
                }
                Err(e) => {
                    error!("Ошибка сброса отметок: {}", e);
                    bot.send_message(chat_id, "⚠️ Не удалось сбросить отметки.").await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_text(
    bot: Bot,
    msg: Message,
    db: DbConnection,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let user_id = msg.from().as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    let text = msg.text().unwrap_or("").trim();

    if text.is_empty() {
        return Ok(());
    }

    let is_reg = match check_registration(&db, user_id).await {
        Ok(v) => v,
        Err(e) => {
            error!("Ошибка проверки регистрации: {}", e);
            bot.send_message(chat_id, "⚠️ Не удалось проверить регистрацию.").await?;
            return Ok(());
        }
    };

    if is_reg {
        bot.send_message(chat_id, "ℹ️ Вы уже зарегистрированы.\nИспользуйте /list.").await?;
        return Ok(());
    }

    match repo::register_student(&db, user_id, text).await {
        Ok(_) => {
            bot.send_message(chat_id, format!("✅ Вы зарегистрированы как {}", text)).await?;
            // После регистрации показываем список (пользователь не админ)
            show_attendance(&bot, chat_id, &db, false, user_id).await?;
        }
        Err(e) => {
            error!("Ошибка регистрации: {}", e);
            bot.send_message(chat_id, "❌ Ошибка регистрации. Попробуйте ещё раз.").await?;
        }
    }
    Ok(())
}

// Вспомогательная функция для обновления основного сообщения со списком
async fn update_attendance_message(
    bot: &Bot,
    msg: &Message,
    db: &DbConnection,
    is_admin: bool,
    current_user_tg_id: i64,
) {
    match repo::get_students_with_today_status(db).await {
        Ok(students) => {
            let (text, keyboard) = render_attendance(&students, is_admin, current_user_tg_id);
            if let Err(e) = bot
                .edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .await
            {
                if !e.to_string().contains("message is not modified") {
                    error!("Ошибка обновления сообщения: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Ошибка получения списка для обновления: {}", e);
        }
    }
}

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    db: DbConnection,
    config: Config,
) -> ResponseResult<()> {
    let data = match q.data {
        Some(data) => data,
        None => return Ok(()),
    };
    let msg = match q.message {
        Some(msg) => msg,
        None => return Ok(()),
    };
    let user_id = q.from.id.0 as i64;
    let username = q.from.username.as_deref();
    let is_admin = is_admin(username, &config);
    let current_user_tg_id = user_id;

    match data.as_str() {
        "refresh" => {
            update_attendance_message(&bot, &msg, &db, is_admin, current_user_tg_id).await;
            bot.answer_callback_query(&q.id).await?;
        }
        "toggle_self" => {
            if let Err(e) = repo::toggle_presence(&db, user_id).await {
                error!("Ошибка переключения статуса: {}", e);
                bot.answer_callback_query(&q.id)
                    .text("❌ Не удалось изменить статус.")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
            update_attendance_message(&bot, &msg, &db, is_admin, current_user_tg_id).await;
            bot.answer_callback_query(&q.id).await?;
        }
        "show_student_selector" => {
            if !is_admin {
                bot.answer_callback_query(&q.id)
                    .text("❌ У вас нет прав администратора.")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
            match repo::get_students_with_today_status(&db).await {
                Ok(students) => {
                    let (text, keyboard) = render_student_selector(&students);
                    if let Err(e) = bot
                        .edit_message_text(msg.chat.id, msg.id, text)
                        .reply_markup(keyboard)
                        .await
                    {
                        if !e.to_string().contains("message is not modified") {
                            error!("Ошибка открытия меню выбора: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Ошибка получения списка: {}", e);
                    bot.answer_callback_query(&q.id)
                        .text("⚠️ Не удалось загрузить список.")
                        .show_alert(true)
                        .await?;
                }
            }
            bot.answer_callback_query(&q.id).await?;
        }
        "back_to_main" => {
            update_attendance_message(&bot, &msg, &db, is_admin, current_user_tg_id).await;
            bot.answer_callback_query(&q.id).await?;
        }
        _ => {
            // Обработка admin_toggle_ (переключение в подменю)
            if let Some(tg_id_str) = data.strip_prefix("admin_toggle_") {
                let tg_id = match tg_id_str.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        bot.answer_callback_query(&q.id)
                            .text("❌ Некорректный ID пользователя.")
                            .show_alert(true)
                            .await?;
                        return Ok(());
                    }
                };
                if !is_admin {
                    bot.answer_callback_query(&q.id)
                        .text("❌ У вас нет прав администратора.")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
                if let Err(e) = repo::toggle_presence(&db, tg_id).await {
                    error!("Ошибка переключения статуса: {}", e);
                    bot.answer_callback_query(&q.id)
                        .text("❌ Не удалось изменить статус.")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
                // Обновляем подменю, оставаясь в нём
                match repo::get_students_with_today_status(&db).await {
                    Ok(students) => {
                        let (text, keyboard) = render_student_selector(&students);
                        if let Err(e) = bot
                            .edit_message_text(msg.chat.id, msg.id, text)
                            .reply_markup(keyboard)
                            .await
                        {
                            if !e.to_string().contains("message is not modified") {
                                error!("Ошибка обновления подменю: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Ошибка получения списка для подменю: {}", e);
                    }
                }
                bot.answer_callback_query(&q.id).await?;
            }
            // Обычное переключение (для не-админа в основном меню)
            else if let Some(tg_id_str) = data.strip_prefix("toggle_") {
                let tg_id = match tg_id_str.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        bot.answer_callback_query(&q.id)
                            .text("❌ Некорректный ID пользователя.")
                            .show_alert(true)
                            .await?;
                        return Ok(());
                    }
                };
                if !is_admin && user_id != tg_id {
                    bot.answer_callback_query(&q.id)
                        .text("⚠️ Вы можете отмечать только себя.")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
                if let Err(e) = repo::toggle_presence(&db, tg_id).await {
                    error!("Ошибка переключения статуса: {}", e);
                    bot.answer_callback_query(&q.id)
                        .text("❌ Не удалось изменить статус.")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
                update_attendance_message(&bot, &msg, &db, is_admin, current_user_tg_id).await;
                bot.answer_callback_query(&q.id).await?;
            }
        }
    }

    Ok(())
}

async fn show_attendance(
    bot: &Bot,
    chat_id: ChatId,
    db: &DbConnection,
    is_admin: bool,
    current_user_tg_id: i64,
) -> ResponseResult<()> {
    match repo::get_students_with_today_status(db).await {
        Ok(students) => {
            let (text, keyboard) = render_attendance(&students, is_admin, current_user_tg_id);
            bot.send_message(chat_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            error!("Ошибка получения списка: {}", e);
            bot.send_message(chat_id, "⚠️ Не удалось загрузить список. Попробуйте позже.")
                .await?;
        }
    }
    Ok(())
}