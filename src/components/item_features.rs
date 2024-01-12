use std::future::Future;
use futures::future::BoxFuture;
use hivemq_openapi::models::Schema;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use crate::action::Item;


pub trait DeleteFn: Send + Sync {
    fn delete(&self, host: String, id: String) -> BoxFuture<'static, Result<String, String>>;
}

impl<T, F> DeleteFn for T
    where
        T: Fn(String, String) -> F + Sync + Send,
        F: Future<Output=Result<String, String>> + 'static + Send,
{
    fn delete(&self, host: String, id: String) -> BoxFuture<'static, Result<String, String>> {
        Box::pin(self(host, id))
    }
}


pub trait ListFn: Send + Sync {
    fn list(&self, host: String) -> BoxFuture<'static, Result<Vec<(String, Item)>, String>>;
}

impl<T, F> ListFn for T
    where
        T: Fn(String) -> F + Sync + Send,
        F: Future<Output=Result<Vec<(String, Item)>, String>> + 'static + Send,
{
    fn list(&self, host: String) -> BoxFuture<'static, Result<Vec<(String, Item)>, String>> {
        Box::pin(self(host))
    }
}

pub trait CreateFn: Send + Sync {
    fn create(&self, host: String, item: String) -> BoxFuture<'static, Result<Item, String>>;
}

impl<T, F> CreateFn for T
    where
        T: Fn(String, String) -> F + Sync + Send,
        F: Future<Output=Result<Item, String>> + 'static + Send,
{
    fn create(&self, host: String, item: String) -> BoxFuture<'static, Result<Item, String>> {
        Box::pin(self(host, item))
    }
}

pub trait ItemSelector<T> {
    fn select(&self, item: Item) -> Option<T>;

    fn select_with_id(&self, item: Item) -> Option<(String, T)>;
}