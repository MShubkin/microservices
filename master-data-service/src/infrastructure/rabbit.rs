use amqprs::BasicProperties;
use rabbit_services::ROUTING_QUEUE;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{error, info, Instrument};

use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
};
use broker::rabbit::{RabbitAdapter, RabbitConsumer};
use broker::{BrokerAdapter, Consumer, Publisher};

use rabbit_services::{
    properties::get_rabbit_span, MASTER_DATA_ACTION_QUEUE, REQUEST_DICTIONARY_QUEUE,
};
use shared_essential::presentation::dto::master_data::{
    error::MasterDataResult, request::MasterDataAction,
};
use shared_essential::presentation::dto::{AsezError, AsezResult};

use crate::application::action;
use crate::infrastructure::GlobalConfig;
use crate::presentation::handlers::rabbit::process_dictionary_request;

macro_rules! handle_action {
    ($fun: expr, $dto: expr, $properties: expr, $config: expr) => {{
        let config = $config.clone();
        let span = get_rabbit_span(&$properties);

        tokio::spawn(
            async move {
                let res = $fun($dto, config.db_pool.as_ref()).await;

                let reply_to = $properties.reply_to().map(|s| s.as_str());
                if let Err(error) = publish_response(res, reply_to, config).await {
                    error!(kind = "tcp", "Publish error: {}", error);
                }
            }
            .instrument(span),
        );
    }};
}

pub async fn start_rabbit_listener(
    config: Arc<GlobalConfig>,
) -> MasterDataResult<()> {
    let mut dictionary_consumer =
        register_consumer(&config.broker_adapter, REQUEST_DICTIONARY_QUEUE).await?;
    let mut action_consumer =
        register_consumer(&config.broker_adapter, MASTER_DATA_ACTION_QUEUE).await?;
    let mut routing_consumer =
        register_consumer(&config.broker_adapter, ROUTING_QUEUE).await?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                dictionary_request = dictionary_consumer.consume() => {
                    match dictionary_request {
                        Ok(rabbit_message) => {
                            let config = config.clone();
                            let span = get_rabbit_span(&rabbit_message.properties);

                            tokio::spawn(
                                async move {
                                    let (dto, reply_to) = (rabbit_message.content, rabbit_message.properties.reply_to().map(|s| s.as_str()));
                                    let res = process_dictionary_request(dto).await;
                                    if let Err(error) =
                                        publish_response(res, reply_to, config).await
                                    {
                                        error!(kind = "tcp", "Publish error: {}", error);
                                    }
                                }
                                .instrument(span),
                            );
                        }
                        Err(err) => {
                            error!(kind = "tcp", "Error consuming message: {:?}", err);
                        }
                    }
                }
                action_request = action_consumer.consume() => {
                    match action_request {
                        Ok(rabbit_message) => {
                            info!(kind = "master_data", "Процессинг действия: {:?}", &rabbit_message);
                            match rabbit_message.content {
                                MasterDataAction::RouteStart(dto) => handle_action!(action::route_start, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteUpdate(dto) => handle_action!(action::route_update, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteStop(dto) => handle_action!(action::route_stop, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteRemove(dto) => handle_action!(action::route_remove, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteCopy(dto) => handle_action!(action::route_copy, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteList(dto) => handle_action!(action::get_route_list, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteDetails(dto) => handle_action!(action::get_route_details, dto, rabbit_message.properties, &config),
                                MasterDataAction::RouteCreate(dto) => handle_action!(action::route_create, dto, rabbit_message.properties, &config),
                                MasterDataAction::OrganizationUserAssignmentSearchById(dto) => handle_action!(action::org_user_assignment_search_by_id, dto, rabbit_message.properties, &config),
                                MasterDataAction::SearchPlanReasonCancel(dto) => handle_action!(action::plan_reasons_cancel_search, dto, rabbit_message.properties, &config),
                                MasterDataAction::OrganizationUserAssignmentSearchByDepartment(dto) => handle_action!(action::org_user_assignment_search_by_department, dto, rabbit_message.properties, &config),
                            }
                        }
                        Err(err) => {
                            error!(kind = "tcp", "Ошибка при получении сообщения на экшен: {:?}", err);
                        }
                    }
                }
                routing_request = routing_consumer.consume() => {
                    match routing_request {
                        Ok(rabbit_message) => {
                            info!(kind = "master_data", "Процессинг поиска маршрута: {:?}", &rabbit_message);
                            handle_action!(action::route_find, rabbit_message.content, rabbit_message.properties, &config);
                        }
                        Err(err) => {
                            error!(kind = "tcp", "Ошибка при получении сообщения на поиск маршрута автоназначения: {:?}", err);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn declare_queues(adapter: &RabbitAdapter) -> MasterDataResult<()> {
    adapter
        .declare_queue(
            QueueDeclareArguments::default()
                .queue(REQUEST_DICTIONARY_QUEUE.into())
                .durable(true)
                .finish(),
        )
        .await?;
    adapter
        .declare_queue(
            QueueDeclareArguments::default()
                .queue(ROUTING_QUEUE.into())
                .durable(true)
                .finish(),
        )
        .await?;
    adapter
        .declare_queue(
            QueueDeclareArguments::default()
                .queue(MASTER_DATA_ACTION_QUEUE.into())
                .durable(true)
                .finish(),
        )
        .await?;
    Ok(())
}

// Generalized function to register `price-analysis` consumer
pub async fn register_consumer(
    adapter: &RabbitAdapter,
    queue_name: &str,
) -> MasterDataResult<RabbitConsumer> {
    let consumer = adapter
        .register_consumer(BasicConsumeArguments::new(
            queue_name,
            "masterdata-consumer",
        ))
        .await?;
    Ok(consumer)
}

pub(crate) async fn publish_response<R>(
    search_result: MasterDataResult<R>,
    reply_to: Option<&str>,
    config: Arc<GlobalConfig>,
) -> MasterDataResult<()>
where
    R: Debug + Serialize + Send + Sync,
{
    let asez_result: AsezResult<R> = search_result.map_err(AsezError::new);

    let reply_to = reply_to.unwrap_or_default();
    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .with_persistence(true)
        .finish();

    let publish_props = BasicPublishArguments::new("", reply_to);

    let basic_publisher = config
        .broker_adapter
        .register_publisher(basic_props, publish_props)
        .await?;

    info!(kind = "master_data", "Отправление ответа: {:?}", &asez_result);

    basic_publisher.publish(&asez_result).await?;

    Ok(())
}
