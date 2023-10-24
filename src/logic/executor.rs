use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_channel::Sender;
use tokio::time::timeout;

use crate::core::error::{Error, ErrorKind};

pub type Executor<LogicRequestType, StorageRequestType> = Arc<
    dyn Fn(
            LogicRequestType,
            Sender<StorageRequestType>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
        + Send
        + Sync,
>;

pub async fn timeout_send_storage_request<StorageRequestType, OkResultType>(
    timeout_after_milliseconds: u64,
    storage_request: StorageRequestType,
    sender: &Sender<StorageRequestType>,
    api_replier: tokio::sync::oneshot::Sender<Result<OkResultType, Error>>,
) -> Result<tokio::sync::oneshot::Sender<Result<OkResultType, Error>>, Error> {
    match timeout(
        Duration::from_millis(timeout_after_milliseconds),
        sender.send(storage_request),
    )
    .await
    {
        Ok(result) => match result {
            Ok(_) => (),
            Err(error) => {
                let error = Error::new(
                    ErrorKind::LogicError,
                    format!("failed to send storage request: {}", &error),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error");
                }

                return Err(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::LogicError,
                format!("timed out sending storage request: {}", &error),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error");
            }

            return Err(error);
        }
    }

    Ok(api_replier)
}

pub async fn timeout_receive_storage_response<StorageOkResultType, LogicOkResultType>(
    timeout_after_milliseconds: u64,
    storage_receiver: tokio::sync::oneshot::Receiver<Result<StorageOkResultType, Error>>,
    api_replier: tokio::sync::oneshot::Sender<Result<LogicOkResultType, Error>>,
) -> Result<
    (
        tokio::sync::oneshot::Sender<Result<LogicOkResultType, Error>>,
        StorageOkResultType,
    ),
    Error,
> {
    let ok_result = match timeout(
        Duration::from_millis(timeout_after_milliseconds),
        storage_receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(ok_result) => ok_result,
                Err(error) => {
                    let error = Error::new(
                        ErrorKind::LogicError,
                        format!("storage failed to handle request: {}", &error),
                    );

                    if let Err(_) = api_replier.send(Err(error.clone())) {
                        log::warn!("failed to reply to api with an error");
                    }

                    return Err(error);
                }
            },
            Err(error) => {
                let error = Error::new(
                    ErrorKind::LogicError,
                    format!("failed to receive response from storage: {}", &error),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error")
                }

                return Err(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::LogicError,
                format!("timed out receiving response from storage: {}", &error),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error");
            }

            return Err(error);
        }
    };

    Ok((api_replier, ok_result))
}
