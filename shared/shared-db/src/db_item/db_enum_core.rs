/// Получение всех вариантов энума вместе с их дискриминантами
/// Автоматически реализуется для енума в момент определения #[derive(DbEnum)]
pub trait EnumDiscriminant: Sized + 'static {
    const DISCRIMINANTS: &'static [(Self, i16)];
}
