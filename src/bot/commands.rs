use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Показать список посещаемости")]
    List,
    #[command(description = "Помощь")]
    Help,
    #[command(description = "Запустить бота (регистрация)")]
    Start,
    #[command(description = "Статистика за сегодня (админ)")]
    Stats,
    #[command(description = "Удалить студента по tg_id (админ)")]
    Remove { tg_id: i64 },
    #[command(description = "Очистить все отметки сегодняшнего дня (админ)")]
    ResetToday,
}