#![allow(dead_code)]

use std::sync::Arc;
use std::sync::Mutex as StdMutex;



pub trait PollableClientFactory<Client> : Send + Sync {
    fn build_client(&self) -> Client;
}

pub struct ClientsPool<Client> {
    clients: StdMutex<Vec<Box<Client>>>,
    factory: Box<dyn PollableClientFactory<Client> + Send + Sync>,
}

pub struct ClientGuard<Client> {
    client: Option<Box<Client>>,
    pool: Arc<ClientsPool<Client>>,
}

impl<Client> Drop for ClientGuard<Client>
{
     fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.return_client(client);
        }
    }
}

impl<Client> ClientsPool<Client> {
    pub fn new(factory: Box<dyn PollableClientFactory<Client>>) -> Self {
        Self {
            clients: StdMutex::new(Vec::new()),
            factory: factory
        }
    }

    pub fn pop_client(self: &Arc<Self>) -> ClientGuard<Client> {
        let mut clients = self.clients.lock().unwrap();
        if clients.len() == 0 {
            return ClientGuard {
                client: Some(Box::new(self.factory.build_client())),
                pool: Arc::clone(self)
            };
        }

        let client = clients.pop().unwrap();
        ClientGuard { client: Some(client), pool: Arc::clone(self) }
    }

    pub fn return_client(&self, client: Box<Client>) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.push(client);
        }
    }
}
