use std::{thread::sleep, time::Duration};

use alloy::signers::local::PrivateKeySigner;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ClientTrigger,
    ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, OrderGrouping,
};
use log::info;

#[tokio::main]
async fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let hyperliquid_private_key = std::env::var("HYPERLIQUID_PRIVATE_KEY").unwrap();
    let wallet: PrivateKeySigner = hyperliquid_private_key.parse().unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // 複数の注文を作成
    let orders = vec![
        ClientOrderRequest {
            asset: "ETH".to_string(),
            is_buy: true,
            reduce_only: false,
            limit_px: 3900.0,
            sz: 0.01,
            cloid: None,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Gtc".to_string(),
            }),
        },
        ClientOrderRequest {
            asset: "ETH".to_string(),
            is_buy: false,
            reduce_only: true,
            limit_px: 4000.0,
            sz: 0.01,
            cloid: None,
            order_type: ClientOrder::Trigger(ClientTrigger {
                is_market: false,
                trigger_px: 4000.0,
                tpsl: "tp".to_string(),
            }),
        },
        ClientOrderRequest {
            asset: "ETH".to_string(),
            is_buy: false,
            reduce_only: true,
            limit_px: 3000.0,
            sz: 0.01,
            cloid: None,
            order_type: ClientOrder::Trigger(ClientTrigger {
                is_market: false,
                trigger_px: 3000.0,
                tpsl: "sl".to_string(),
            }),
        },
    ];

    // OrderGrouping::NormalTpsl でバルク注文を送信
    info!("Sending bulk order with OrderGrouping::NormalTpsl");
    let response = exchange_client
        .bulk_order_with_grouping(orders, None, OrderGrouping::NormalTpsl)
        .await
        .unwrap();
    info!("Bulk order with NormalTpsl grouping placed: {response:?}");

    // 最初の注文の結果を確認
    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };

    if let Some(data) = response.data {
        for (i, status) in data.statuses.iter().enumerate() {
            info!("Order {} status: {:?}", i + 1, status);

            // 注文が成功した場合、キャンセルを試行
            if let Some(oid) = match status {
                ExchangeDataStatus::Filled(order) => Some(order.oid),
                ExchangeDataStatus::Resting(order) => Some(order.oid),
                _ => None,
            } {
                // 少し待ってからキャンセル
                sleep(Duration::from_secs(30));

                let cancel = ClientCancelRequest {
                    asset: "ETH".to_string(),
                    oid,
                };

                let cancel_response = exchange_client.cancel(cancel, None).await.unwrap();
                info!("Order {} cancelled: {cancel_response:?}", i + 1);
            }
        }
    }
}
