// https://github.com/CleverCloud/lapin

use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};

use tardis::basic::result::TardisResult;
use tardis::config::config_dto::{CacheConfig, DBConfig, FrameworkConfig, MQConfig, MQModuleConfig, MailConfig, OSConfig, SearchConfig, TardisConfig, WebServerConfig};
use tardis::test::test_container::TardisTestContainer;
use tardis::TardisFuns;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[tokio::test(flavor = "multi_thread")]
async fn test_mq_client() -> TardisResult<()> {
    env::set_var("RUST_LOG", "info,tardis=trace");
    // enable when debug tokio
    // console_subscriber::init();
    TardisFuns::init_log()?;
    TardisTestContainer::rabbit(|url| async move {
        // Default test
        TardisFuns::init_conf(TardisConfig {
            cs: Default::default(),
            fw: FrameworkConfig {
                app: Default::default(),
                web_server: WebServerConfig {
                    enabled: false,
                    ..Default::default()
                },
                web_client: Default::default(),
                cache: CacheConfig {
                    enabled: false,
                    ..Default::default()
                },
                db: DBConfig {
                    enabled: false,
                    ..Default::default()
                },
                mq: MQConfig {
                    enabled: true,
                    url: url.clone(),
                    modules: HashMap::from([("m1".to_string(), MQModuleConfig { url: url.clone() })]),
                },
                search: SearchConfig {
                    enabled: false,
                    ..Default::default()
                },
                mail: MailConfig {
                    enabled: false,
                    ..Default::default()
                },
                os: OSConfig {
                    enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .await?;

        TardisFuns::mq();
        let client = TardisFuns::mq_by_module("m1");
        client
            .response("test-addr", move |(header, msg)| async move {
                println!("response1 {}", msg);
                // tokio current thread runtime + tokio await point + lapin may block tokio task(only polled a few times)
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await?;

        client
            .response("test-addr", |(header, msg)| async move {
                println!("response2 {}", msg);
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await?;

        client
            .subscribe("test-topic", |(header, msg)| async move {
                println!("subscribe1 {}", msg);
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await?;

        client
            .subscribe("test-topic", |(header, msg)| async move {
                println!("subscribe2 {}", msg);
                assert_eq!(header.get("k1").unwrap(), "v1");
                assert_eq!(msg, "测试!");
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await?;

        let mut header = HashMap::new();
        header.insert("k1".to_string(), "v1".to_string());

        client.request("test-addr", "测试!".to_string(), &header).await?;
        client.request("test-addr", "测试!".to_string(), &header).await?;
        client.request("test-addr", "测试!".to_string(), &header).await?;
        client.request("test-addr", "测试!".to_string(), &header).await?;

        client.publish("test-topic", "测试!".to_string(), &header).await?;
        client.publish("test-topic", "测试!".to_string(), &header).await?;
        client.publish("test-topic", "测试!".to_string(), &header).await?;
        client.publish("test-topic", "测试!".to_string(), &header).await?;

        loop {
            if COUNTER.load(Ordering::SeqCst) == 12 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                break;
            }
        }

        client.close().await?;
        Ok(())
    })
    .await
}
