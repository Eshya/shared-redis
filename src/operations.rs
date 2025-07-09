use crate::config::{get_redis_pool, AsyncConnManager};
use anyhow::Result as AnyResult;
use redis::aio::PubSub;
use redis::AsyncCommands;
use redis::{ExistenceCheck, SetOptions};
use std::env;
use std::marker::{Send, Sync};

pub async fn broadcasting_data(db_channel: String, data: String) -> AnyResult<()> {
    let mut connection = get_redis_pool().await.unwrap();
    let _: () = connection.publish(db_channel, data).await.unwrap();
    Ok(())
}

pub async fn subscribe_data(db_channel: String) -> AnyResult<PubSub> {
    let connection = get_redis_pool().await.unwrap();
    let mut pubsub = connection.into_pubsub();
    pubsub.subscribe(db_channel).await.unwrap();
    Ok(pubsub)
}

pub async fn set_if_not_exist<T>(key: String, data: T, mut conn: AsyncConnManager) -> AnyResult<bool>
where
    T: 'static + Clone + Sync + Send + redis::ToRedisArgs,
{
    let res = conn.set_nx(key, data).await.unwrap();

    Ok(res)
}

pub async fn get_data<T>(key: String, mut conn: AsyncConnManager) -> AnyResult<Option<T>>
where
    T: redis::FromRedisValue,
{
    let res = conn.get(key).await.ok();
    let result = match res {
        Some(res) => Some(res),
        None => None,
    };
    Ok(result)
}

pub async fn set_data<T>(key: String, data: T, mut conn: AsyncConnManager) -> AnyResult<bool>
where
    T: 'static + Clone + Sync + Send + redis::ToRedisArgs,
{
    let res = conn.set(key, data).await.unwrap();

    Ok(res)
}

pub async fn set_with_options<T>(key: String, data: T, mut conn: AsyncConnManager) -> AnyResult<bool>
where
    T: 'static + Clone + Sync + Send + redis::ToRedisArgs,
{
    let expiry_in_sec = env::var("IDEMPOTENT_EXPIRY_IN_SEC").unwrap_or("120".to_string()).parse().unwrap_or(120);
    let opts = SetOptions::default().conditional_set(ExistenceCheck::NX).with_expiration(redis::SetExpiry::EX(expiry_in_sec));
    let res = conn.set_options(key, data, opts).await.unwrap();

    Ok(res)
}
