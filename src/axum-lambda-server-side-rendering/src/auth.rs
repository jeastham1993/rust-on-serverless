pub mod auth {
    use aws_sdk_dynamodb::{model::AttributeValue, Client};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    pub struct AuthService {
        client: Client,
        table_name: String,
    }

    impl AuthService {
        pub fn new(client: Client, table_name: String) -> AuthService {
            AuthService {
                client: client,
                table_name: table_name,
            }
        }

        pub async fn generate_session(&self) -> String {
            tracing::debug!("Generating session");

            let session_token = Uuid::new_v4().to_string().to_uppercase();

            let duration = Duration::from_secs(300);

            let start = SystemTime::now().checked_add(duration).unwrap();
            let epoch_time = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            tracing::debug!("Expiring on {}", epoch_time.as_secs());
            tracing::debug!("Storing in {}", &self.table_name);

            let res = &self
                .client
                .put_item()
                .table_name(&self.table_name)
                .item(
                    "PK",
                    AttributeValue::S(format!("SESSION#{}", session_token.to_uppercase())),
                )
                .item(
                    "SK",
                    AttributeValue::S(format!("SESSION#{}", session_token.to_uppercase())),
                )
                .item("TTL", AttributeValue::N(epoch_time.as_secs().to_string()))
                .send()
                .await;

            tracing::debug!("Stored session data");

            session_token
        }

        pub async fn validate_sesssion(&self, token: String) -> Result<(), ()> {
            let res = &self
                .client
                .get_item()
                .table_name(&self.table_name)
                .key(
                    "PK",
                    AttributeValue::S(format!("SESSION#{}", token.to_uppercase())),
                )
                .key(
                    "SK",
                    AttributeValue::S(format!("SESSION#{}", token.to_uppercase())),
                )
                .send()
                .await;

            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }
}
