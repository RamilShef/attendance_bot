use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "students")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub tg_id: i64,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::attendance::Entity")]
    Attendance,
}

impl Related<super::attendance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attendance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}