use super::*;

#[test]
fn test_even_port_string() -> Result<()> {
    let mut p = EvenPort::default();
    assert_eq!(
        p.to_string(),
        "reserve: false",
        "bad value {} for reselve: false",
        p
    );

    p.reserve_port = true;
    assert_eq!(
        p.to_string(),
        "reserve: true",
        "bad value {} for reselve: true",
        p
    );

    Ok(())
}

#[test]
fn test_even_port_false() -> Result<()> {
    let mut m = Message::new();
    let p = EvenPort {
        reserve_port: false,
    };
    p.add_to(&mut m)?;
    m.write_header();

    let mut decoded = Message::new();
    let mut port = EvenPort::default();
    decoded.write(&m.raw)?;
    port.get_from(&m)?;
    assert_eq!(port, p);

    Ok(())
}

#[test]
fn test_even_port_add_to() -> Result<()> {
    let mut m = Message::new();
    let p = EvenPort { reserve_port: true };
    p.add_to(&mut m)?;
    m.write_header();
    //"GetFrom"
    {
        let mut decoded = Message::new();
        decoded.write(&m.raw)?;
        let mut port = EvenPort::default();
        port.get_from(&decoded)?;
        assert_eq!(port, p, "Decoded {}, expected {}", port, p);

        //"HandleErr"
        {
            let mut m = Message::new();
            let mut handle = EvenPort::default();
            if let Err(err) = handle.get_from(&m) {
                assert!(
                    stun::error::Error::ErrAttributeNotFound.equal(&err),
                    "{} should be not found",
                    err
                );
            }
            m.add(ATTR_EVEN_PORT, &[1, 2, 3]);
            if let Err(err) = handle.get_from(&m) {
                assert!(
                    is_attr_size_invalid(&err),
                    "IsAttrSizeInvalid should be true"
                );
            } else {
                assert!(false, "expected error, but got ok");
            }
        }
    }

    Ok(())
}
