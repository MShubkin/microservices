pub mod plan_amendment;

use shared_essential::presentation::dto::response_request::Message;

pub trait Extract<C> {
    fn extract(&self) -> Option<C>;
}

#[allow(dead_code)]
trait Validator<T> {
    /// Имеет ли валидатор на текущий момент ошибки
    fn has_errors(&self) -> bool;

    /// Валидна ли сущность с определенным айди
    fn is_valid(&self, entity: &T) -> bool;

    /// Пометить сущность как невалидную
    fn mark_invalid(&mut self, entity: &T, msg: Message);

    /// Валидация каждого элемента по отдельности
    fn for_each<E, F, ErrFn>(&mut self, validate_fn: F, err_fn: ErrFn)
    where
        T: Extract<E>,
        F: Fn(E) -> bool,
        ErrFn: Fn(&T) -> Message;

    /// Валидация элементов в совокупности
    fn all<E, F>(&mut self, validate_fn: F, msg: Message)
    where
        T: Extract<E>,
        F: Fn(E) -> bool;

    /// Возвращение только валидных элементов
    fn finalise(self) -> Vec<T>;
}
