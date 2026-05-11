use shared_essential::{
    domain::{EcAgenda, EcProtocol, EcProtocolItem, PlanOrAmendment},
    presentation::dto::{
        processing::ActionType,
        response_request::{BusinessMessage, Message},
    },
};

#[derive(Debug)]
pub enum PlanApproveByChiefMessage {
    Success,
    Refunded,
    FieldIsMissing(&'static str),
}

#[derive(Debug)]
pub enum PlanApproveMessage<'a> {
    Success,
    InvalidPlanStatus,
    AlreadyInProtocolWarn(&'a EcProtocol),
    AlreadyInProtocolErr(&'a EcProtocol),
}

#[derive(Debug)]
pub enum PlanReturnToExpertMessage<'a> {
    Success,
    InvalidPlanStatus,
    AlreadyInProtocolWarn(&'a EcProtocol),
    AlreadyInProtocolErr(&'a EcProtocol),
}

#[derive(Debug)]
pub enum PlanReturnToCustomerMessage<'a> {
    Success(ActionType),
    AlreadyInProtocolWarn(&'a EcProtocol),
    AlreadyInProtocolErr(&'a EcProtocol),
    InvalidPlanStatus,
}

#[derive(Debug)]
pub enum PlanCancelMessage<'a> {
    Success,
    InvalidPlanStatus,
    AlreadyInProtocolWarn(&'a EcProtocol),
    AlreadyInProtocolErr(&'a EcProtocol),
    #[allow(dead_code)]
    MissingCancelReason,
    #[allow(dead_code)]
    MissingIsNewPlan,
}

#[derive(Debug)]
pub enum PlanChangeFormMessage<'a> {
    CorrespondenceSuccess,
    NoCommissionSuccess,
    InPersonSuccess,
    InvalidProtocolResult(&'a EcProtocol, &'a EcProtocolItem),
    AlreadyInAgenda(&'a EcAgenda),
    AlreadyInProtocolWarn(&'a EcProtocol),
    AlreadyInProtocolErr(&'a EcProtocol),
    InvalidPlanStatus,
}

#[derive(Debug)]
pub enum PlanChangeCommissionDateMessage<'a> {
    Success,
    AlreadyInAgenda(&'a EcAgenda),
    InvalidPlanStatus,
    AlreadyInProtocol(&'a EcProtocol),
}

impl BusinessMessage for PlanApproveByChiefMessage {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success => {
                Message::success(format!("Цена ППЗ/ДС {} определена", entity.id()))
            }
            Self::Refunded => Message::info(format!(
                "ППЗ/ДС {} отправлена Заказчику на доработку",
                entity.id()
            )),
            Self::FieldIsMissing(field) => Message::error(format!(
                "У ППЗ/ДС {} отсутствует поле {field}",
                entity.id()
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success => Message::success(format!(
                "По {} ППЗ/ДС цена определена",
                entities.len()
            )),
            Self::Refunded => Message::info(format!(
                "{} ППЗ/ДС отправлено Заказчику на доработку",
                entities.len()
            )),
            Self::FieldIsMissing(field) => Message::error(format!(
                "У ППЗ/ДС {} отсутствует поле {field}",
                entities.len()
            )),
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for PlanApproveMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success => {
                Message::success(format!("Вы утвердили ППЗ/ДС {}", entity.id()))
            }

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Вы хотите утвердить. Подтвердить?"#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить утверждение невозможно. ППЗ/ДС {} находится не на статусах СК", 
                entity.id()
            )),

            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Выполнить утверждение невозможно."#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success => {
                Message::success(format!("Вы утвердили {} ППЗ/ДС", entities.len()))
            }

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Вы хотите утвердить. Подтвердить?"#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить утверждение невозможно. {} ППЗ/ДС находятся не на статусах СК", 
                entities.len()
            )),

            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Выполнить утверждение невозможно."#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for PlanReturnToExpertMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success => {
                Message::success(format!("Вы вернули Эксперту АЦ ППЗ/ДС {}", entity.id()))
            }

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Вы хотите вернуть Эксперту АЦ. Подтвердить?"#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить возврат Эксперту АЦ невозможно. ППЗ/ДС {} находится не на статусах СК.", 
                entity.id()
            )),

            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Выполнить возврат Эксперту невозможно."#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success => {
                Message::success(format!("Вы вернули Эксперту АЦ {} ППЗ/ДС", entities.len()))
            }

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Вы хотите вернуть Эксперту АЦ. Подтвердить?"#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить возврат Эксперту АЦ невозможно. {} ППЗ/ДС находятся не на статусах СК",
                entities.len()
            )),
            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Выполнить возврат Эксперту невозможно."#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for PlanReturnToCustomerMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success(action) => {
                match action {
                    ActionType::Revision => Message::success(format!(
                        "Вы вернули Заказчику на доработку ППЗ/ДС {}",
                        entity.id()
                    )),
                    ActionType::Documentation => Message::success(format!(
                        "Вы вернули Заказчику на запрос документации ППЗ/ДС {}",
                        entity.id()
                    )),
                }
            },

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Вы хотите вернуть Заказчику. Подтвердить?"#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить возврат Заказчику невозможно. ППЗ/ДС {} находится не на статусах СК", 
                entity.id()
            )),
            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Вернуть Заказчику невозможно."#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success(action) => match action {
                ActionType::Revision => Message::success(format!(
                    "Вы вернули Заказчику на доработку {} ППЗ/ДС",
                    entities.len()
                )),
                ActionType::Documentation => Message::success(format!(
                    "Вы вернули Заказчику на запрос документации {} ППЗ/ДС",
                    entities.len()
                )),
            },

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Вы хотите вернуть Заказчику. Подтвердить?"#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить возврат Заказчику невозможно. {} ППЗ/ДС находится не на статусах СК", 
                entities.len()
            )),
            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Вернуть Заказчику невозможно."#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for PlanCancelMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success => Message::success(format!("Вы аннулировали ППЗ/ДС {}", entity.id())),

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Вы хотите аннулировать. Подтвердить?"#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить аннулирование невозможно. ППЗ/ДС {} находится не на статусах СК",
                entity.id()
            )),

            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Выполнить аннулирование невозможно."#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
            Self::MissingCancelReason => Message::error(format!(
                "Для ППЗ/ДС {} не заполнено обязательное поле «Причина аннулирования»",
                entity.id()
            )),
            Self::MissingIsNewPlan =>  Message::error(
                "Заполните обязательное поле «Номер новой ППЗ/ ДС».".to_string()
            ),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success => Message::success(format!(
                "Вы аннулировали {} ППЗ/ДС", 
                entities.len()
            )),

            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Вы хотите аннулировать. Подтвердить?"#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить аннулирование невозможно. {} ППЗ/ДС находится не на статусах СК", 
                entities.len()
            )),
            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Выполнить аннулирование невозможно."#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
            Self::MissingCancelReason => Message::error(format!(
                "Для ППЗ/ДС {} не заполнено обязательное поле «Причина аннулирования»",
                entities.len()
            )),
            Self::MissingIsNewPlan =>  Message::error(
                "Заполните обязательное поле «Номер новой ППЗ/ ДС».".to_string()
            ),
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for PlanChangeFormMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::CorrespondenceSuccess => Message::success(format!(
                "Вы перевели на заочную СК ППЗ/ДС {}", 
                entity.id()
            )),
            Self::NoCommissionSuccess => Message::success(format!(
                "Вы приняли решение, что СК не требуется по ППЗ/ДС {}", 
                entity.id()
            )),
            Self::InPersonSuccess => Message::success(format!(
                "Вы перевели на очную СК ППЗ/ДС {}", 
                entity.id()
            )),

            Self::AlreadyInAgenda(agenda) => Message::warn(format!(
                r#"ППЗ/ДС {} включена в Повестку {} на {} в статусе "{}". Вы хотите изменить форму СК. Подтвердить?"#, 
                entity.id(), agenda.id, agenda.meeting_date, agenda.status_id
            ))
            .with_param_item(*agenda),
            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} в статусе "{}". Вы хотите изменить форму СК. Подтвердить?"#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidProtocolResult(protocol, protocol_item) => {
                Message::error(format!(
                    r#"ППЗ/ДС {} включена в Протокол {} от {} с решением "{}". Изменить форму СК невозможно"#, 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol_item.result_id
                ))
                .with_param_item(*protocol)
            }
            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    "ППЗ/ДС {} включена в Протокол {} от {} в статусе {}. Выполнить изменение формы СК невозможно.", 
                    entity.id(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить изменение формы СК невозможно. ППЗ/ДС {} находится не на статусах СК.",
                entity.id()
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::CorrespondenceSuccess => Message::success(format!(
                "Вы перевели на заочную СК {} ППЗ/ДС", 
                entities.len()
            )),
            Self::NoCommissionSuccess => Message::success(format!(
                "Вы приняли решение, что СК не требуется по {} ППЗ/ДС", 
                entities.len()
            )),
            Self::InPersonSuccess => Message::success(format!(
                "Вы перевели на очную СК {} ППЗ/ДС", 
                entities.len()
            )),

            Self::AlreadyInAgenda(agenda) => Message::warn(format!(
                r#"{} ППЗ/ДС включены в Повестку {} на {} в статусе "{}". Вы хотите изменить форму СК. Подтвердить?"#, 
                entities.len(), agenda.id, agenda.meeting_date, agenda.status_id
            ))
            .with_param_item(*agenda),
            Self::AlreadyInProtocolWarn(protocol) => {
                Message::warn(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Вы хотите изменить форму СК. Подтвердить?"#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },

            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить изменение формы СК невозможно. {} ППЗ/ДС находятся не на статусах СК.", 
                entities.len()
            )),
            Self::InvalidProtocolResult(protocol, _) => {
                Message::error(format!(
                    "{} ППЗ/ДС включены в Протокол {} от {} с неподходящим решением. Изменить форму СК невозможно", 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date
                ))
                .with_param_item(*protocol)
            }
            Self::AlreadyInProtocolErr(protocol) => {
                Message::error(format!(
                    r#"{} ППЗ/ДС включены в Протокол {} от {} в статусе "{}". Выполнить изменение формы СК невозможно."#, 
                    entities.len(),
                    protocol.id,
                    protocol.protocol_date,
                    protocol.status_id
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for PlanChangeCommissionDateMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success => Message::success(format!(
                "Вы изменили дату очной СК по ППЗ/ДС {}",
                entity.id()
            )),

            Self::AlreadyInAgenda(agenda) => Message::warn(format!(
                "ППЗ/ДС {} включена в Повестку {} на {}. Вы подтверждаете изменение даты очной СК?",
                entity.id(), agenda.id, agenda.meeting_date
            ))
            .with_param_item(*agenda),

            Self::AlreadyInProtocol(protocol) => Message::error(format!(
                "Выполнить изменение даты очной СК невозможно. ППЗ/ДС {} включена в Протокол {} от {}",
                entity.id(), protocol.id, protocol.protocol_date
            ))
            .with_param_item(*protocol),
            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить изменение даты очной СК невозможно. ППЗ/ДС {} находится не на статусах СК",
                entity.id()
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success => Message::success(format!(
                "Вы изменили дату очной СК по {} ППЗ/ДС",
                entities.len()
            )),

            Self::AlreadyInAgenda(agenda) => Message::warn(format!(
                "{} ППЗ/ДС включены в Повестку {} на {}. Вы подтверждаете изменение даты очной СК?",
                entities.len(), agenda.id, agenda.meeting_date
            ))
            .with_param_item(*agenda),

            Self::AlreadyInProtocol(protocol) => Message::error(format!(
                "Выполнить изменение даты очной СК невозможно. {} ППЗ/ДС включены в Протокол {} от {}",
                entities.len(), protocol.id, protocol.protocol_date
            ))
            .with_param_item(*protocol),
            Self::InvalidPlanStatus => Message::error(format!(
                "Выполнить изменение даты очной СК невозможно. {} ППЗ/ДС находятся не на статусах СК",
                entities.len()
            )),
        };

        msg.with_param_items(entities)
    }
}
