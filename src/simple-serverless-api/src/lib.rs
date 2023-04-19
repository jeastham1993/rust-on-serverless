use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use mockall::automock;

#[automock]
#[async_trait]
pub trait DataAccess {
    async fn create(&self, id: String, payload: String) -> Result<(), ()>;
    async fn get(&self, id: String) -> Result<String, ()>;
    async fn delete(&self, id: String) -> Result<(), ()>;
}

pub struct DynamoDbDataAccess {
    client: Client,
    table_name: String,
}

impl DynamoDbDataAccess {
    pub fn new(client: Client, table_name: String) -> DynamoDbDataAccess {
        DynamoDbDataAccess {
            client: client,
            table_name: table_name,
        }
    }
}

#[async_trait]
impl DataAccess for DynamoDbDataAccess {
    async fn create(&self, id: String, payload: String) -> Result<(), ()> {
        let res = &self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("id", AttributeValue::S(id.to_string()))
            .item("payload", AttributeValue::S(payload))
            .send()
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    async fn get(&self, id: String) -> Result<String, ()> {
        let res = &self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await;

        // Return a response to the end-user
        match res {
            Ok(query_result) => {
                let payload = query_result.item().expect("Payload attribute should exist");

                Ok(payload["payload"].as_s().unwrap().to_string().into())
            }
            Err(_) => Err(()),
        }
    }

    async fn delete(&self, id: String) -> Result<(), ()> {
        let res = &self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
