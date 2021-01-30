use super::*;
use crate::relay::none::*;

use util::Error;

use std::net::IpAddr;
use std::str::FromStr;

use tokio::net::UdpSocket;
use tokio::time::{Duration, Instant};

const STATIC_KEY: &str = "ABC";

#[tokio::test]
async fn test_allocation_lifetime_parsing() -> Result<(), Error> {
    let lifetime = Lifetime(Duration::from_secs(5));

    let mut m = Message::new();
    let lifetime_duration = allocation_lifetime(&m);

    assert_eq!(
        lifetime_duration, DEFAULT_LIFETIME,
        "Allocation lifetime should be default time duration"
    );

    lifetime.add_to(&mut m)?;

    let lifetime_duration = allocation_lifetime(&m);
    assert_eq!(
        lifetime_duration, lifetime.0,
        "Expect lifetime_duration is {}, but {:?}",
        lifetime, lifetime_duration
    );

    Ok(())
}

#[tokio::test]
async fn test_allocation_lifetime_overflow() -> Result<(), Error> {
    let lifetime = Lifetime(MAXIMUM_ALLOCATION_LIFETIME * 2);

    let mut m2 = Message::new();
    lifetime.add_to(&mut m2)?;

    let lifetime_duration = allocation_lifetime(&m2);
    assert_eq!(
        lifetime_duration, DEFAULT_LIFETIME,
        "Expect lifetime_duration is {:?}, but {:?}",
        DEFAULT_LIFETIME, lifetime_duration
    );

    Ok(())
}

struct TestAuthHandler;
impl AuthHandler for TestAuthHandler {
    fn auth_handle(
        &self,
        _username: &str,
        _realm: &str,
        _src_addr: SocketAddr,
    ) -> Result<Vec<u8>, Error> {
        Ok(STATIC_KEY.as_bytes().to_vec())
    }
}

#[tokio::test]
async fn test_allocation_lifetime_deletion_zero_lifetime() -> Result<(), Error> {
    //env_logger::init();

    let l = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

    let allocation_manager = Manager::new(ManagerConfig {
        relay_addr_generator: Box::new(RelayAddressGeneratorNone {
            address: "0.0.0.0".to_owned(),
        }),
    });

    let socket = SocketAddr::new(IpAddr::from_str("127.0.0.1")?, 5000);

    let mut r = Request::new(l, socket, allocation_manager, Box::new(TestAuthHandler {}));

    {
        let mut nonces = r.nonces.lock().await;
        nonces.insert(STATIC_KEY.to_owned(), Instant::now());
    }

    let five_tuple = FiveTuple {
        src_addr: r.src_addr,
        dst_addr: r.conn.local_addr()?,
        protocol: PROTO_UDP,
    };

    r.allocation_manager
        .create_allocation(
            five_tuple.clone(),
            Arc::clone(&r.conn),
            0,
            Duration::from_secs(3600),
        )
        .await?;
    assert!(r
        .allocation_manager
        .get_allocation(&five_tuple)
        .await
        .is_some());

    let mut m = Message::new();
    Lifetime::default().add_to(&mut m)?;
    MessageIntegrity(STATIC_KEY.as_bytes().to_vec()).add_to(&mut m)?;
    Nonce::new(ATTR_NONCE, STATIC_KEY.to_owned()).add_to(&mut m)?;
    Realm::new(ATTR_REALM, STATIC_KEY.to_owned()).add_to(&mut m)?;
    Username::new(ATTR_USERNAME, STATIC_KEY.to_owned()).add_to(&mut m)?;

    r.handle_refresh_request(&m).await?;
    assert!(r
        .allocation_manager
        .get_allocation(&five_tuple)
        .await
        .is_none());

    Ok(())
}