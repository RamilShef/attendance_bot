use anyhow::Result;
use chrono::Local;
use sea_orm::{
    prelude::*,
    ActiveModelTrait,
    ColumnTrait,
    DatabaseConnection,
    DbErr,
    EntityTrait,
    QueryFilter,
    QueryOrder,
    QuerySelect,
    Set,
};

use sea_orm::sea_query::OnConflict;

use super::entities::{students, attendance};

pub async fn is_student_registered(db: &DatabaseConnection, tg_id: i64) -> Result<bool> {
    let exists = students::Entity::find()
        .filter(students::Column::TgId.eq(tg_id))
        .one(db)
        .await?
        .is_some();
    Ok(exists)
}

pub async fn register_student(db: &DatabaseConnection, tg_id: i64, name: &str) -> Result<()> {
    let new_student = students::ActiveModel {
        tg_id: Set(tg_id),
        name: Set(name.to_string()),
        ..Default::default()
    };
    new_student.insert(db).await?;
    Ok(())
}

pub async fn remove_student(db: &DatabaseConnection, tg_id: i64) -> Result<()> {
    if let Some(student) = students::Entity::find()
        .filter(students::Column::TgId.eq(tg_id))
        .one(db)
        .await?
    {
        student.delete(db).await?;
    }
    Ok(())
}

pub async fn reset_today_attendance(db: &DatabaseConnection) -> Result<()> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    attendance::Entity::delete_many()
        .filter(attendance::Column::Date.eq(today))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn get_students_with_today_status(db: &DatabaseConnection) -> Result<Vec<(i64, String, bool)>> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let students_list = students::Entity::find()
        .order_by_asc(students::Column::Name)
        .all(db)
        .await?;

    let mut result = Vec::new();
    for student in students_list {
        let present = attendance::Entity::find()
            .filter(attendance::Column::StudentId.eq(student.id))
            .filter(attendance::Column::Date.eq(&today))
            .one(db)
            .await?
            .map(|a| a.present)
            .unwrap_or(false);
        result.push((student.tg_id, student.name, present));
    }
    Ok(result)
}

pub async fn toggle_presence(db: &DatabaseConnection, tg_id: i64) -> Result<bool> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let student = students::Entity::find()
        .filter(students::Column::TgId.eq(tg_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Student not found"))?;

    let existing = attendance::Entity::find()
        .filter(attendance::Column::StudentId.eq(student.id))
        .filter(attendance::Column::Date.eq(&today))
        .one(db)
        .await?;

    let new_present = if let Some(record) = existing {
        !record.present
    } else {
        true
    };

    let active_model = attendance::ActiveModel {
        student_id: Set(student.id),
        date: Set(today),
        present: Set(new_present),
    };
    attendance::Entity::insert(active_model)
        .on_conflict(
            OnConflict::columns([
                attendance::Column::StudentId,
                attendance::Column::Date,
            ])
                .update_column(attendance::Column::Present)
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(new_present)
}

pub async fn get_stats(db: &DatabaseConnection) -> Result<(i64, i64)> {
    let total = students::Entity::find().count(db).await? as i64;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let present = attendance::Entity::find()
        .filter(attendance::Column::Date.eq(today))
        .filter(attendance::Column::Present.eq(true))
        .count(db)
        .await? as i64;
    Ok((total, present))
}

pub async fn set_all_present(db: &DatabaseConnection) -> Result<(), DbErr> {
    let students = students::Entity::find().all(db).await?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    for student in students {
        let attendance = attendance::ActiveModel {
            student_id: Set(student.id),
            date: Set(today.clone()),
            present: Set(true),
        };
        attendance::Entity::insert(attendance)
            .on_conflict(
                OnConflict::columns([
                    attendance::Column::StudentId,
                    attendance::Column::Date,
                ])
                    .update_column(attendance::Column::Present)
                    .to_owned(),
            )
            .exec(db)
            .await?;
    }
    Ok(())
}