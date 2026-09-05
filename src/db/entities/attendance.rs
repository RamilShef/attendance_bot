use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "attendance")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub student_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub date: String,
    pub present: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::students::Entity",
        from = "Column::StudentId",
        to = "super::students::Column::Id"
    )]
    Student,
}

impl Related<super::students::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Student.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}