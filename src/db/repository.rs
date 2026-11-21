use crate::error::AppError;

pub trait Repository<T: Send + Sync + Clone> {
    fn find_all(&self) -> Result<Vec<T>, AppError>;
    fn find_by_id(&self, id: &str) -> Result<Option<T>, AppError>;
    fn insert(&self, entity: T) -> Result<T, AppError>;
    fn delete_by_id(&self, id: &str) -> Result<bool, AppError>;
}
