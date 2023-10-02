## ARPStore SDK

A SDK for ARPStore API.

### Usage

```rust
use arpstore_sdk::Client;

const PRODUCT_CODE: &str = "the_product_code";

#[tokio::main]
async fn main() {
    let arp_client = Client::new("https://api.yourdomain.com", "your_subscription_key");
    let result = arp_client.is_valid_subscription(PRODUCT_CODE).await;
    match result {
        Ok(_) => println!("Subscription is valid"),
        Err(e) => println!("{e}"), // The error is a string from the API, or a reqwest error
    }
}
```

### License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
