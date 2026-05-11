//! Модуль для экспорта частей приложения для интеграционных
//! тестов
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
#[cfg(test)]
pub mod tests;
